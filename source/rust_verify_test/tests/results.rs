#![feature(rustc_private)]
#[macro_use]
mod common;
use common::*;

test_verify_one_file! {
    #[test] test_result verus_code! {
        use vstd::prelude::*;

        struct Err {
            error_code: int,
        }

        // Result::unwrap and Result::unwrap_err require trait bounds

        use core::fmt::Debug;

        #[verifier::external]
        impl Debug for Err {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                unimplemented!();
            }
        }

        proof fn test_result() {
            let ok_result = Result::<i8, Err>::Ok(1);
            assert(ok_result is Ok);
            assert(ok_result.unwrap() == 1);
            let err_result = Result::<i8, Err>::Err(Err{ error_code: -1 });
            assert(err_result is Err);
            assert(err_result->Err_0 == Err{ error_code: -1 });
        }
    } => Ok(())
}

test_verify_one_file! {
    #[test] test_result_fails verus_code! {
        use vstd::prelude::*;

        struct Err {
            error_code: int,
        }

        proof fn test_ok_result() {
            let ok_result = Result::<int, Err>::Ok(1);
            assert(ok_result is Err); // FAILS
        }

        proof fn test_err_result() {
            let err_result = Result::<int, Err>::Err(Err{ error_code: -1 });
            assert(err_result is Ok); // FAILS
        }
    } => Err(err) => assert_fails(err, 2)
}

test_verify_one_file! {
    #[test] test_result_expect verus_code! {
        use vstd::prelude::*;

        struct Err {
            error_code: int,
        }

        // Result::unwrap and Result::unwrap_err require trait bounds

        use core::fmt::Debug;

        #[verifier::external]
        impl Debug for Err {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
                unimplemented!();
            }
        }

        proof fn test_result() {
            let ok_result = Result::<i8, Err>::Ok(1);
            assert(ok_result is Ok);
            assert(ok_result.expect("the result is ok") == 1);
            let err_result = Result::<i8, Err>::Err(Err{ error_code: -1 });
            assert(err_result is Err);
            assert(err_result->Err_0 == Err{ error_code: -1 });
        }
    } => Ok(())
}
