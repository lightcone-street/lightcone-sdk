// Auth domain — nonce + ed25519 signed-message login, session, logout. Pure
// namespace: each alias re-exports its implementation module, whose `.resi` is
// the real interface.

module Method = Auth__Model.Method
module ChainType = Auth__Model.ChainType
module PrivyEmbeddedWallet = Auth__Model.PrivyEmbeddedWallet
module PrivyData = Auth__Model.PrivyData
module XAccount = Auth__Model.XAccount
module GoogleAccount = Auth__Model.GoogleAccount
module Identity = Auth__Model.Identity
module User = Auth__Model.User
module Session = Auth__Model.Session
module SignedLogin = Auth__Model.SignedLogin
module Raw = Auth__Raw
module Native = Auth__Native
module Client = Auth__Client

module EmailAccount = Auth__Model.EmailAccount

module LinkedIdentity = Auth__Model.LinkedIdentity

module LinkedIdentitySelector = Auth__Model.LinkedIdentitySelector

module RegisterPrivyRequest = Auth__Model.RegisterPrivyRequest

module Credentials = Auth__Model.Credentials

module RegisterPrivyConflict = Auth__Model.RegisterPrivyConflict

let tradingWallet: (User.t, Method.t) => string
let credentialsFromSession: Session.t => Credentials.t
let walletDisplayName: (User.t, Method.t) => string
let displayName: User.t => string
let avatarUrl: User.t => option<string>
let classifyRegisterPrivyConflict: SdkError.t => option<RegisterPrivyConflict.t>
