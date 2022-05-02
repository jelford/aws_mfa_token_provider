This repository is companion to a pull request that adds MFA support to the AWS Rust SDK.

It provides a use-case for the new `ProviderMfaToken` and a possible implementation for a provider
that sources mfa tokens from stdin.

The original intention was to include something like this stdin provide into the main PR, but
given that I've had to bring in a couple of crates for password handling, maybe that was too
bullish.
