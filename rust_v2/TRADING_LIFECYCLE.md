```
                            +------------------+
                            | Collateral (USDC)|
                            +--------+---------+
                                     |
               +---------------------+---------------------+
               v                                           v
    +---------------------+                     +---------------------+
    |  deposit_to_global  |                     |  mint_complete_set  |
    |                     |                     |                     |
    | Collateral -> Global|                     | Collateral -> Market|
    |  Deposit Account    |                     | (single tx)         |
    +----------+----------+                     +----------+----------+
               v                                           |
    +----------------------+                               |
    |global_to_market_dep. |                               |
    |                      |                               |
    | Global -> Market,    |                               |
    | mint cond. tokens    |                               |
    +----------+-----------+                               |
               +---------------------+---------------------+
                                     v
                          +---------------------+
                          | Position Account    |
                          |                     |
                          | Holds conditional   |
                          | tokens (one per     |
                          | outcome) in market  |
                          +----------+----------+
                                     |
======================================================================
                              TRADING LOOP
======================================================================
                                     |
                                     v
                  +-------------------------------------+
                  |         Construct & Sign Order      |
                  |                                     |  OFF-CHAIN
                  |  OrderBuilder / SignedOrder         |  (client)
                  |  Ed25519 sign over keccak256 hash   |
                  +------------------+------------------+
                                     |
                                     v
                  +-------------------------------------+
                  |        Submit to Matching Engine    |
                  |                                     |  OFF-CHAIN
                  |  client.orders().submit()           |  (API)
                  |  Order enters the orderbook         |
                  +------------------+------------------+
                                     |
                          +----------+----------+
                          |                     |
                          v                     v
                  +--------------+     +--------------+
                  | No Match     |     | Orders Cross |
                  |              |     |              |
                  | Rests on     |     | Engine finds |  OFF-CHAIN
                  | orderbook    |     | matching     |  (matching
                  | until filled |     | orders       |   engine)
                  | or cancelled |     |              |
                  +--------------+     +------+-------+
                                              |
                                              v
                  +-------------------------------------+
                  |        On-Chain Settlement          |
                  |                                     |  ON-CHAIN
                  |  MatchOrdersMulti instruction       |  (Solana)
                  |  1. Verify Ed25519 signatures       |
                  |  2. Validate order parameters       |
                  |  3. Transfer conditional tokens     |
                  |     between maker <-> taker         |
                  |  4. Update fill amounts             |
                  +-------------------------------------+
                                     |
======================================================================
                               EXIT PATHS
======================================================================
                                     |
          +--------------+-----------+-----------+---------------+
          v              v           v           v               v
   +-------------+ +----------+ +--------+ +----------+ +------------+
   |Cancel Order | |Cancel All| |Incr.   | |  Merge   | |  Redeem    |
   |             | |          | |Nonce   | | Complete | |  Winnings  |
   | Remove from | | Remove   | |        | |   Set    | |            |
   | orderbook   | | all from | |Inval.  | |          | | After mkt  |
   | (off-chain) | | orderbook| |all     | | Burn all | | settlement |
   |             | |(off-chain| |orders  | | outcome  | | burn       |
   |             | |  + scope | |below   | | tokens,  | | winning    |
   |             | |  to book)| |nonce   | | recover  | | tokens,    |
   |             | |          | |(on-    | | collat.  | | receive    |
   |             | |          | |chain)  | | (on-     | | collateral |
   |             | |          | |        | |  chain)  | | (on-chain) |
   +-------------+ +----------+ +--------+ +----+-----+ +-----+------+
                                                |             |
                                                v             v
                                           +----------------------+
                                           |  Collateral returned |
                                           |  to user wallet      |
                                           +----------------------+
```
