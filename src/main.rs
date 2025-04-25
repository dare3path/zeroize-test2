//use rustls_pki_types::{PrivatePkcs8KeyDer, PrivateKeyDer};
//use zeroize::Zeroize;
//use zeroize::{Zeroize, DefaultIsZeroes};
//use anyhow::Context;

//fn assert_zeroize<T: Zeroize + DefaultIsZeroes>() {}
//fn assert_zeroize<T: zeroize::Zeroize>() {}
//const _: () = {
//    assert_zeroize::<rustls_pki_types::PrivatePkcs8KeyDer<'static>>();
//    assert_zeroize::<rustls_pki_types::PrivateKeyDer<'static>>();
//};

//#[allow(dead_code)]
//const fn assert_zeroize0<T: Zeroize>() {}
//const _: () = {
//    assert_zeroize0::<PrivatePkcs8KeyDer<'static>>();
//    assert_zeroize0::<PrivateKeyDer<'static>>();
//};

// Assert that rustls-pki-types implements Zeroize.
// Fails if using v1.11.0; use >= 1.12.0 or git = "https://github.com/rustls/pki-types.git".
// Note: v1.11.0 uses ZeroizeOnDrop via Drop, which is sufficient for load_private_key.

#[cfg(not(rustls_pki_types_zeroize))]
compile_error!("build.rs didn't run");

macro_rules! assert_has_zeroize {
    ($t:ty) => {
        const _: () = {
            #[allow(dead_code)]
            const fn check_has_zeroize<T: zeroize::Zeroize>() {}
            check_has_zeroize::<$t>();
        };
//        #[cfg(no_rustls_pki_types_zeroize)]
//        const _: () = {
//            compile_error!(concat!(
//                "The crate 'rustls-pki-types' does not implement zeroize::Zeroize",
//                " for ", stringify!($t),
//                ". Try bumping to version >= 1.12.0 or use git = \"https://github.com/rustls/pki-types.git\", rev = \"b59e08d49911b10c423d25bd9040cfbe5a6042ff\" (at least)."
//            )
//            );
//        };
    };
}

//#[cfg(rustls_pki_types_zeroize)]
assert_has_zeroize!(rustls_pki_types::PrivatePkcs8KeyDer<'static>);
//#[cfg(rustls_pki_types_zeroize)]
assert_has_zeroize!(rustls_pki_types::PrivateKeyDer<'static>);

//macro_rules! assert_zeroize {
//    ($t:ty) => {
//        // Use a trait bound to check if the type implements Zeroize
//        const _: () = {
//            fn check<T: Zeroize>() {}
//            fn assert() {
//                check::<$t>();
//            }
//        };
//    };
//    ($t:ty, $msg:literal) => {
//        // If the type doesn't implement Zeroize, this will fail to compile
//        const _: () = {
//            fn check<T: Zeroize>() {}
//            fn assert() {
//                check::<$t>();
//            }
////            #[allow(dead_code)]
////            struct AssertZeroize;
////            impl AssertZeroize {
////                const MSG: &'static str = $msg;
////            }
//            // Trigger compile_error! if Zeroize is not implemented
//            #[cfg(not(feature = "rustls_pki_types_has_zeroize"))]
//            compile_error!($msg); //AssertZeroize::MSG);
//        };
//    };
//}
//
//// Assert that PrivatePkcs8KeyDer and PrivateKeyDer implement Zeroize
//assert_zeroize!(
//    PrivatePkcs8KeyDer<'static>,
//    "rustls-pki-types does not implement Zeroize for PrivatePkcs8KeyDer. Try bumping to a version >= 1.12.0."
//);
//assert_zeroize!(
//    PrivateKeyDer<'static>,
//    "rustls-pki-types does not implement Zeroize for PrivateKeyDer. Try bumping to a version >= 1.12.0."
//);

fn main() {
    println!("Zeroize assert passed!");
}

#[cfg(test)]
mod tests {
//    use super::*;
    use rustls_pemfile;
    use tokio::io::AsyncReadExt;
    use rustls_pki_types::{PrivatePkcs8KeyDer, PrivateKeyDer};
    use zeroize::Zeroize;

    #[test]
    fn test_zeroize_integration() {
        fn assert_zeroize1<T: zeroize::Zeroize>() {}
        assert_zeroize1::<rustls_pki_types::PrivatePkcs8KeyDer<'static>>();
        assert_zeroize1::<rustls_pki_types::PrivateKeyDer<'static>>();
    }


    #[tokio::test]
    async fn test_zeroize_key() {
        // Read private.pem
        let mut file = tokio::fs::File::open("private.pem")
            .await
            .expect("Failed to open private.pem");
        let mut pem_data = Vec::new();
        file.read_to_end(&mut pem_data)
            .await
            .expect("Failed to read private.pem");

        // Parse key
        let mut cursor = std::io::Cursor::new(&pem_data);
        let mut keys: Vec<PrivatePkcs8KeyDer<'static>> = rustls_pemfile::pkcs8_private_keys(&mut cursor)
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to parse private key");
        assert!(!keys.is_empty(), "No private key found in private.pem");
        assert!(keys.len() == 1, "more than 1 private keys in file private.pem");

//        // Test Zeroize
//        let mut key = PrivateKeyDer::Pkcs8(keys.pop().unwrap());
//        key.zeroize(); // Should compile if Zeroize is implemented
        // Test Zeroize on PrivatePkcs8KeyDer
        let mut pkcs8_key = keys[0].clone_key(); // Clone to avoid moving
        pkcs8_key.zeroize();

        // Test Zeroize on PrivateKeyDer
        //let mut key = PrivateKeyDer::Pkcs8(keys.into_iter().next().unwrap());
        let mut key = PrivateKeyDer::Pkcs8(keys.pop().unwrap());
        key.zeroize();


        // Zeroize pem_data
        pem_data.zeroize();
    }
}
