mod ignore {
    use bstr::{BStr, ByteSlice};
    use git_attributes::{Ignore, Match, MatchGroup};
    use git_glob::pattern::Case;
    use std::io::Read;

    struct Expectations<'a> {
        lines: bstr::Lines<'a>,
    }

    impl<'a> Iterator for Expectations<'a> {
        type Item = (&'a BStr, Option<(&'a BStr, usize, &'a BStr)>);

        fn next(&mut self) -> Option<Self::Item> {
            let line = self.lines.next()?;
            let (left, value) = line.split_at(line.find_byte(b'\t').unwrap());
            let value = value[1..].as_bstr();

            let source_and_line = if left == b"::" {
                None
            } else {
                let mut tokens = left.split(|b| *b == b':');
                let source = tokens.next().unwrap().as_bstr();
                let line_number: usize = tokens.next().unwrap().to_str_lossy().parse().ok().unwrap();
                let pattern = tokens.next().unwrap().as_bstr();
                Some((source, line_number, pattern))
            };
            Some((value, source_and_line))
        }
    }

    #[test]
    fn from_git_dir() {
        let dir =
            git_testtools::scripted_fixture_repo_read_only("make_global_and_external_and_dir_ignores.sh").unwrap();
        let repo_dir = dir.join("repo");
        let git_dir = repo_dir.join(".git");
        let baseline = std::fs::read(git_dir.parent().unwrap().join("git-check-ignore.baseline")).unwrap();
        let mut buf = Vec::new();
        let mut group = MatchGroup::from_git_dir(git_dir, Some(dir.join("user.exclude")), &mut buf).unwrap();
        assert_eq!(
            group.add_patterns_file("not-a-file", None).unwrap(),
            false,
            "missing files are no problem and cause a negative response"
        );
        assert!(
            group
                .add_patterns_file(repo_dir.join(".gitignore"), repo_dir.as_path().into())
                .unwrap(),
            "existing files return true"
        );

        buf.clear();
        let ignore_file = repo_dir.join("dir-with-ignore").join(".gitignore");
        std::fs::File::open(&ignore_file)
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        group.add_patterns_buffer(&buf, ignore_file, repo_dir.as_path().into());

        for (path, source_and_line) in (Expectations {
            lines: baseline.lines(),
        }) {
            let actual = group.pattern_matching_relative_path(
                path,
                repo_dir.join(path.to_str_lossy().as_ref()).is_dir(),
                Case::Sensitive,
            );
            match (actual, source_and_line) {
                (
                    Some(Match {
                        sequence_number,
                        pattern: _,
                        source,
                        value: _,
                    }),
                    Some((expected_source, line, _expected_pattern)),
                ) => {
                    assert_eq!(sequence_number, line, "our counting should match the one used in git");
                    assert_eq!(
                        source.map(|p| p.canonicalize().unwrap()),
                        Some(
                            repo_dir
                                .join(expected_source.to_str_lossy().as_ref())
                                .canonicalize()
                                .unwrap()
                        )
                    );
                }
                (None, None) => {}
                (actual, expected) => panic!("actual {:?} should match {:?} with path '{}'", actual, expected, path),
            }
        }
    }

    #[test]
    fn from_overrides() {
        let input = ["simple", "pattern/"];
        let group = git_attributes::MatchGroup::<Ignore>::from_overrides(input);
        assert_eq!(
            group.pattern_matching_relative_path("Simple", false, git_glob::pattern::Case::Fold),
            Some(pattern_to_match(&git_glob::parse("simple").unwrap(), 0))
        );
        assert_eq!(
            group.pattern_matching_relative_path("pattern", true, git_glob::pattern::Case::Sensitive),
            Some(pattern_to_match(&git_glob::parse("pattern/").unwrap(), 1))
        );
        assert_eq!(group.patterns.len(), 1);
        assert_eq!(
            git_attributes::PatternList::<Ignore>::from_overrides(input),
            group.patterns.into_iter().next().unwrap()
        );
    }

    fn pattern_to_match(pattern: &git_glob::Pattern, sequence_number: usize) -> Match<'_, ()> {
        Match {
            pattern,
            value: &(),
            source: None,
            sequence_number,
        }
    }
}
