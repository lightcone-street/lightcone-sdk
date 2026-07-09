// Faucet domain — testnet SOL + whitelisted deposit-token claims. Pure
// namespace: each alias re-exports its Faucet__* implementation file, whose
// `.resi` is the real interface.

module Raw = Faucet__Raw
module Client = Faucet__Client
