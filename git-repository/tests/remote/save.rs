mod save_as_to {
    use crate::basic_repo;
    use git_repository as git;
    use std::convert::TryInto;

    #[test]
    fn anonymous_remotes_cannot_be_save_lacking_a_name() -> crate::Result {
        let repo = basic_repo()?;
        let remote = repo.remote_at("https://example.com/path")?;
        assert!(matches!(
            remote.save_to(&mut git::config::File::default()).unwrap_err(),
            git::remote::save::Error::NameMissing { .. }
        ));
        Ok(())
    }

    #[test]
    fn new_anonymous_remote_with_name() -> crate::Result {
        let repo = basic_repo()?;
        let mut remote = repo
            .remote_at("https://example.com/path")?
            .push_url("https://ein.hub/path")?;
        let remote_name = "origin";
        assert!(
            repo.find_remote(remote_name).is_err(),
            "there is no remote of that name"
        );
        assert_eq!(remote.name(), None);
        let mut config = git::config::File::default();
        remote.save_as_to(remote_name.try_into().expect("valid name"), &mut config)?;
        assert_eq!(
            uniformize(config.to_string()),
            "[remote \"origin\"]\n\turl = https://example.com/path\n\tpushurl = https://ein.hub/path\n"
        );
        Ok(())
    }

    fn uniformize(input: String) -> String {
        input.replace("\r\n", "\n")
    }
}
