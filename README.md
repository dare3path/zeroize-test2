This is meant to fail-fast from `build.rs` if `rustls-pki-types` crate doesn't implement `zeroize::Zeroize` for two private key types.  
  
To test how this works when it doesn't have zeroize, comment out the patch section in `Cargo.toml`  
Otherwise, it's using the git version which does have it and won't fail on build.  
