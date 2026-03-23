use gtfsfabrik::commands::init;

// TODO: write tests for init !!! (trying TDD as recommended by rust book)

// TEST 1: successfully creates main fabrik directory
#[test]
fn create_fabrik_no_args() {
    let desired_fabrik_name = "/home/aaron/projects/gtfsfabrik/testing_fabriks/test";
}
// TEST 2: successfully creates all subdirectories

// TEST 3: successfully creates .state.toml file and fabrik.toml file (depends on some future
// design choices)

// TEST 4: proper user feedback for invalid paths, and fancy shmancy menus
