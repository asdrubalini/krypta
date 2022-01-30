use fs::PathFinder;
use rand::{prelude::SmallRng, SeedableRng};
use tmp::{RandomFill, Tmp};

const EXPECTED_CHECKSUMS: [u64; 50] = [
    971903, 779540, 1472649, 1051213, 1286672, 1226054, 1039396, 1343095, 1021241, 1546616, 965630,
    1699478, 1033030, 1539104, 1076755, 1954028, 1293802, 1512970, 1525371, 1034934, 1246064,
    1588534, 874099, 1327276, 1428849, 1440590, 1455701, 1053762, 1746618, 2033541, 1658345,
    1216939, 1505234, 1540076, 1434443, 1540939, 1137936, 1611442, 1846618, 1753061, 1268051,
    1337574, 1603575, 1539444, 1318198, 1892841, 961648, 1962485, 1672905, 1567012,
];

#[test]
fn test_path_finder() {
    let mut rng = SmallRng::seed_from_u64(4);

    for (i, files_count) in (100..150).enumerate() {
        let tmp = Tmp::random();
        tmp.random_fill(files_count, &mut rng);

        let path_finder = PathFinder::from_source_path(tmp.base_path()).unwrap();

        assert_eq!(path_finder.metadatas.len(), files_count);

        let len_checksum: u64 = path_finder
            .metadatas
            .iter()
            .map(|(_, metadata)| metadata.len())
            .sum();

        assert_eq!(len_checksum, EXPECTED_CHECKSUMS[i]);
    }
}
