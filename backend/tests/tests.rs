use backend::backend::Backend;
use serial_test::serial;

#[test]
fn series_titles() {
    let backend = Backend::new();
    let series: Vec<String> = backend.view();

    assert_eq!(
        vec![
            "Akiba Maid Wars",
            "Bocchi the Rock",
            "Girls Last Tour",
            "Hanasaku Iroha"
        ],
        series
    );
}

#[test]
#[serial]
fn serie_episodes() {
    let backend = Backend::new();
    let episodes: Vec<String> = backend.titles[0].view();

    let base_name = backend.titles[0].name.as_str();
    let mut episodes_test: Vec<String> = Vec::new();

    for i in 1..4 {
        episodes_test.push(format!("{base_name} - 0{i}.mkv"));
    }

    assert_eq!(episodes_test, *episodes)
}

#[test]
#[serial]
fn open_episode() {
    let mut backend = Backend::new();
    let title = &mut backend.titles[0];

    title.load_episodes();
    let ep = &mut title.episodes[0];

    let command = format!(
        "--start={} --end={} \"{}\"",
        ep.metadata.duration - 5.0,
        ep.metadata.duration - 4.0,
        ep.path.display()
    );

    let output = Backend::run_mpv(&command).expect("[ERROR] - Failed to execute process.");
    assert!(output.status.success());

    ep.update_metadata().unwrap();
    assert_eq!(1417.0, ep.metadata.current.ceil());
}

#[test]
#[serial]
fn is_watched() {
    let mut backend = Backend::new();
    let ep = &mut backend.titles[0].episodes[0];

    // Running video with mpv
    let command = format!(
        "--start={} --end={} \"{}\"",
        ep.metadata.duration - 5.0,
        ep.metadata.duration - 4.0,
        ep.path.display()
    );

    let output = Backend::run_mpv(&command).expect("[ERROR] - Failed to execute process.");
    assert!(output.status.success());
    ep.update_metadata().unwrap();
    assert!(ep.metadata.watched);
}
