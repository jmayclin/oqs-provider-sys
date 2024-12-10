pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let ptr = unsafe {oqs_sys::kem::OQS_KEM_alg_identifier(3)};
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
