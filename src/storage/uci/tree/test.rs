    use super::*;

    #[test]
    fn test_uci_load_config() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.name, "uci_config");
        } else {
            panic!("load test_data/uci_config failed");
        };

        let err = tree.load_config("uci_config");
        assert!(err.is_err());

        Ok(())
    }

    #[test]
    fn test_uci_load_config_force() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config_force("uci_config")?;
        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.name, "uci_config");
        } else {
            panic!("load test_data/uci_config failed");
        };

        if let Some(config) = tree.get_config("uci_config") {
            assert_eq!(config.name, "uci_config");
        } else {
            panic!("load test_data/uci_config failed");
        };

        Ok(())
    }

    #[test]
    fn test_uci_commit() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.add_section("uci_config", "section", "new_section")?;
        tree.set_option_values(
            "uci_config",
            "new_section",
            "abc",
            vec![format!("1"), format!("2"), format!("3")],
        )?;
        tree.commit()?;
        tree.load_config_force("uci_config")?;
        let opt_res = tree.get_option_value("uci_config", "new_section", "abc");
        assert!(opt_res.is_ok());
        tree.del_section("uci_config", "new_section")?;
        tree.commit()?;
        Ok(())
    }

    #[test]
    fn test_uci_revert() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.add_section("uci_config", "section", "new_section")?;
        tree.revert(vec!["uci_config".to_owned()])?;
        tree.load_config("uci_config")?;
        if let Some(config) = tree.get_config("uci_config") {
            assert!(!config.modified);
            Ok(())
        } else {
            panic!("revert test_data/uci_config failed");
        }
    }

    #[test]
    fn test_uci_get_sections() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Some(sections) = tree.get_sections("uci_config", "internal") {
            assert_eq!(sections.len(), 4);
            Ok(())
        } else {
            panic!("get sections of 'sec_type: internal' failed");
        }
    }

    #[test]
    fn test_uci_get_option_value() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(values) = tree.get_option_value("uci_config", "main", "lang") {
            assert_eq!(values[0], "auto");
            Ok(())
        } else {
            panic!("get option value of 'uci_config.main.lang' failed");
        }
    }

    #[test]
    fn test_uci_get_last_value() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(values) = tree.get_option_last_value("uci_config", "ntp", "server") {
            if values.is_some() {
                assert_eq!(values.unwrap(), "3.lede.pool.ntp.org");
            }
            Ok(())
        } else {
            panic!("get option last value of 'uci_config.ntp.server' failed");
        }
    }

    #[test]
    fn test_uci_get_bool_value() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if let Ok(values) = tree.get_option_bool_value("uci_config", "ccache", "enable") {
            assert!(values);
            Ok(())
        } else {
            panic!("get option bool value of 'uci_config.ccache.enable' failed");
        }
    }

    #[test]
    fn test_uci_set_option_values() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        if tree
            .set_option_values("uci_config", "main", "lang", vec![format!("en")])
            .is_ok()
        {
            if let Ok(values) = tree.get_option_value("uci_config", "main", "lang") {
                assert_eq!(values[0], format!("en"));
            };
            Ok(())
        } else {
            panic!("set option values of 'uci_config.main.lang=en' failed");
        }
    }

    #[test]
    fn test_uci_del_option() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.del_option("uci_config", "main", "lang")?;
        if tree.get_option_value("uci_config", "main", "lang").is_ok() {
            panic!("delete option of 'uci_config.main.lang' failed");
        } else {
            Ok(())
        }
    }

    #[test]
    fn test_uci_add_section() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.add_section("uci_config", "core", "setting")?;
        tree.set_option_values("uci_config", "setting", "off", vec![])?;
        if tree
            .get_option_value("uci_config", "setting", "off")
            .is_ok()
        {
            Ok(())
        } else {
            panic!("add section of 'uci_config.setting' failed");
        }
    }

    #[test]
    fn test_uci_del_section() -> Result<()> {
        let mut tree = UciTree::new("test_data");
        tree.load_config("uci_config")?;
        tree.del_section("uci_config", "main")?;
        if tree.get_option_value("uci_config", "main", "lang").is_ok() {
            panic!("delete section of 'uci_config.main' failed");
        } else {
            Ok(())
        }
    }