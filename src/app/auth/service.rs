use super::repo;

#[mry::mry]
pub fn join(twice: bool) -> i32 {
    let mut result = repo::join(5);

    if twice {
        result += repo::join(5);
    }

    result + 10
}

#[cfg(test)]
mod tests {
    use super::{repo, *};

    #[test]
    #[mry::lock(repo::join)]
    fn join_calls_repo_join_twice() {
        let mock_join = repo::mock_join(mry::Any).returns(20);

        assert_eq!(join(true), 50);

        // repo::mock_join(5).assert_called(2);
        mock_join.assert_called(2);
    }

    #[test]
    #[mry::lock(repo::join)]
    fn join_calls_repo_join_once() {
        let mock_join = repo::mock_join(mry::Any).returns(20);

        assert_eq!(join(false), 30);

        mock_join.assert_called(1);
    }
}
