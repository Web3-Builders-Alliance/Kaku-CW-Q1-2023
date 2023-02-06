// https://github.com/cosmorama/wynddex/blob/main/contracts/stake/src/stake.rs

pub const CLAIMS: Claims = Claims::new("claims"); // Creates state to track who is owed tokens during "staking"

#[cw_serde]
pub struct Config {
    // Settings that describe the whole contract and can presumabily be used to smooth migrations
    /// address of cw20 contract token to stake
    pub cw20_contract: Addr, // already came with an explanation, but i'm not sure why
    // it's useful or helpful to keep the contract address in state
    /// address that instantiated the contract
    pub instantiator: Addr, // same as above
    pub tokens_per_power: Uint128,
    pub min_bond: Uint128,
    /// configured unbonding periods in seconds
    pub unbonding_periods: Vec<UnbondingPeriod>, // wynd allows different bonding periods which confer different rewards
    /// the maximum number of distributions that can be created
    pub max_distributions: u32,
}

#[cw_serde]
#[derive(Default)]
pub struct BondingInfo {
    /// the amount of staked tokens which are not locked
    stake: Uint128,
    /// Vec of locked_tokens sorted by expiry timestamp
    locked_tokens: Vec<(Timestamp, Uint128)>,
}

impl BondingInfo {
    /// Add an amount of tokens to the stake
    pub fn add_unlocked_tokens(&mut self, amount: Uint128) -> Uint128 {
        let tokens = self.stake.checked_add(amount).unwrap(); // checked add is addition that returns a Result which mainly errors if you overflowed

        self.stake = tokens; // simply do the addition and if it succeds the user now has that many more tokens that are not locked to an expiration

        tokens
    }

    /// Inserts a new locked_tokens entry in its correct place with a provided expires Timestamp and an amount
    pub fn add_locked_tokens(&mut self, expires: Timestamp, amount: Uint128) {
        // Insert the new locked_tokens entry into its correct place using a binary search and an insert
        match self.locked_tokens.binary_search(&(expires, amount)) {
            Ok(pos) => self.locked_tokens[pos].1 += amount, // if the spot in the vec already exists just bump that stored amount in the tuple
            Err(pos) => self.locked_tokens.insert(pos, (expires, amount)), // otherwise make a new spot in the vec for that expiration and token amount
        }
    }

    /// Free any tokens which are now considered unlocked
    /// Split locked tokens based on which are expired and assign the remaining ones to locked_tokens
    /// For each unlocked one, add this amount to the stake
    pub fn free_unlocked_tokens(&mut self, env: &Env) {
        // this is seemingly for performing delegation unbonding in csdk
        if self.locked_tokens.is_empty() {
            return; // cant free tokens if none are locked.
        }
        let (unlocked, remaining): (Vec<_>, Vec<_>) = self
            .locked_tokens
            .iter()
            .partition(|(time, _)| time <= &env.block.time); // partition creates a tuple where the first is the items that passed the predicate and the second is the others
                                                             // here we're grouping based on if the expiration time for the tokens has passed the current block time of this contract execution
        self.locked_tokens = remaining;

        self.stake += unlocked.into_iter().map(|(_, v)| v).sum::<Uint128>(); // sum up the unlocked tokens and add them to the user's existing unlocked count
    }

    /// Attempt to release an amount of stake. First releasing any already unlocked tokens
    /// and then subtracting the requested amount from stake.
    /// On success, returns total_unlocked() after reducing the stake by this amount.
    pub fn release_stake(&mut self, env: &Env, amount: Uint128) -> Result<Uint128, OverflowError> {
        self.free_unlocked_tokens(env);

        let new_stake = self.stake.checked_sub(amount)?;

        self.stake = new_stake;

        Ok(self.stake)
    }

    /// Return all locked tokens at a given block time that is all
    /// locked_tokens with a Timestamp > the block time passed in env as a param
    pub fn total_locked(&self, env: &Env) -> Uint128 {
        let locked_stake = self
            .locked_tokens
            .iter()
            .filter_map(|(t, v)| if t > &env.block.time { Some(v) } else { None }) // check of each locked token expiration against the current block time
            .sum::<Uint128>(); // sum up all of the locked tokens
        locked_stake
    }

    /// Return all locked tokens at a given block time that is all
    /// locked_tokens with a Timestamp > the block time passed in env as a param
    pub fn total_unlocked(&self, env: &Env) -> Uint128 {
        let mut unlocked_stake: Uint128 = self.stake;
        unlocked_stake += self
            .locked_tokens
            .iter()
            .filter_map(|(t, v)| if t <= &env.block.time { Some(v) } else { None })
            .sum::<Uint128>();

        unlocked_stake
    }

    /// Return all stake for this BondingInfo, including locked_tokens
    pub fn total_stake(&self) -> Uint128 {
        let total_stake: Uint128 = self
            .stake
            .checked_add(self.locked_tokens.iter().map(|x| x.1).sum()) //sums all of the tokens that are locked regardless of expiration time
            .unwrap();
        total_stake
    }
}

pub const REWARD_CURVE: Map<&AssetInfoValidated, Curve> = Map::new("reward_curve"); // stores the reward curve for given denoms

pub const ADMIN: Admin = Admin::new("admin");
pub const CONFIG: Item<Config> = Item::new("config"); // stores the config as state

#[derive(Default, Serialize, Deserialize)]
pub struct TokenInfo {
    // how many tokens are fully bonded
    pub staked: Uint128,
    // how many tokens are unbounded and awaiting claim
    pub unbonding: Uint128,
}

impl TokenInfo {
    pub fn total(&self) -> Uint128 {
        self.staked + self.unbonding
    }
}

pub const TOTAL_STAKED: Item<TokenInfo> = Item::new("total_staked"); // state that tracks the total

pub const STAKE: Map<(&Addr, UnbondingPeriod), BondingInfo> = Map::new("stake");

#[derive(Default, Serialize, Deserialize)]
pub struct TotalStake {
    /// Total stake
    pub staked: Uint128,
    /// Total stake minus any stake that is below min_bond by unbonding period.
    /// This is used when calculating the total staking power because we don't
    /// want to count stakes below min_bond into the total.
    pub powered_stake: Uint128,
}
/// Total stake minus any stake that is below min_bond by unbonding period.
/// This is used when calculating the total staking power because we don't
/// want to count stakes below min_bond into the total.
///
/// Using an item here to save some gas.
pub const TOTAL_PER_PERIOD: Item<Vec<(UnbondingPeriod, TotalStake)>> =
    Item::new("total_per_period");

/// Loads the total powered stake of the given period.
/// See [`TOTAL_PER_PERIOD`] for more details.
pub fn load_total_of_period(
    storage: &dyn Storage,
    unbonding_period: UnbondingPeriod,
) -> Result<TotalStake, ContractError> {
    let mut totals = TOTAL_PER_PERIOD.load(storage)?;
    totals
        .binary_search_by_key(&unbonding_period, |(period, _)| *period)
        .map_err(|_| ContractError::NoUnbondingPeriodFound(unbonding_period))
        .map(|idx| totals.swap_remove(idx).1)
}
