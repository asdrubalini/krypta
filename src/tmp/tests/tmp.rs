use rand::{prelude::SmallRng, SeedableRng};
use tmp::Tmp;

#[test]
fn test_temp_path_folder_creation_and_destruction() {
    let mut rng = SmallRng::seed_from_u64(69);

    for _ in 0..256 {
        let path = {
            let tmp = Tmp::random_with_rng(&mut rng);

            // Make sure that path gets created
            assert!(tmp.base_path().exists());
            tmp.base_path()
        };

        // Make sure that path gets destroyed
        
        assert!(!path.exists());
    }
}
