# Crypto Notes

## Supported Format

The first implementation supports authenticated Bitwarden encrypted strings:

```text
2.<base64 iv>|<base64 ciphertext>|<base64 hmac>
```

The symmetric key is the standard 64-byte Bitwarden form:

- bytes `0..32`: AES-256-CBC encryption key
- bytes `32..64`: HMAC-SHA256 authentication key

The HMAC input is `iv || ciphertext`. Decryption verifies the MAC before
attempting AES-CBC decryption.

## Explicitly Disabled

Legacy AES-CBC encrypted strings without a MAC, including type `0` and untyped
`iv|ciphertext` strings, are rejected. This matches the current Bitwarden client
direction and avoids accepting tamperable ciphertext in a Kubernetes secret
sync path.

## Current Scope

This code covers field-level cipher decryption once the correct user,
organization, or item key is already available. Key derivation, user API-key
login, organization key unwrap, and per-cipher key handling are separate
milestones and must receive their own fixture tests.
