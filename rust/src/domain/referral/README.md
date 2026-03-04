# Referrals

Beta access status and referral code management.

[Back to SDK root](../../README.md)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [Examples](#examples)

## Types

### `ReferralStatus`

Current referral and beta access status for the authenticated user.

| Field | Type | Description |
|-------|------|-------------|
| `is_beta` | `bool` | Whether the user has beta access |
| `source` | `Option<String>` | How the user gained access (e.g., referral code, whitelist) |
| `referral_codes` | `Vec<ReferralCodeInfo>` | Referral codes the user can share |

### `ReferralCodeInfo`

A single referral code owned by the user.

| Field | Type | Description |
|-------|------|-------------|
| `code` | `String` | The referral code string |
| `max_uses` | `i32` | Maximum number of redemptions |
| `use_count` | `i64` | Current number of redemptions |

### `RedeemResult`

Result of redeeming a referral code.

| Field | Type | Description |
|-------|------|-------------|
| `success` | `bool` | Whether the redemption succeeded |
| `is_beta` | `bool` | Whether the user now has beta access |

## Client Methods

Access via `client.referrals()`. All methods require an authenticated session.

### `get_status`

```rust
async fn get_status(&self) -> Result<ReferralStatus, SdkError>
```

Get the current user's referral status, including their beta access state and any referral codes they own.

### `redeem`

```rust
async fn redeem(&self, code: &str) -> Result<RedeemResult, SdkError>
```

Redeem a referral code to gain beta access. Returns an error if the code is invalid, expired, or already at max uses.

## Examples

### Check referral status and share codes

```rust
use lightcone::prelude::*;

async fn check_referral(client: &LightconeClient) -> Result<(), SdkError> {
    let status = client.referrals().get_status().await?;

    if status.is_beta {
        println!("Beta access: active");
        for code in &status.referral_codes {
            println!("  Code: {} ({}/{} uses)", code.code, code.use_count, code.max_uses);
        }
    } else {
        println!("No beta access yet. Redeem a code to get started.");
    }

    Ok(())
}
```

### Redeem a referral code

```rust
use lightcone::prelude::*;

async fn redeem_code(client: &LightconeClient, code: &str) -> Result<(), SdkError> {
    let result = client.referrals().redeem(code).await?;
    if result.success {
        println!("Referral code redeemed. Beta access: {}", result.is_beta);
    }
    Ok(())
}
```
