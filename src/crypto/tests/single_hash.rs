use std::{fs::remove_file, path::Path};

use crypto::{hash::Sha256FileHasher, traits::Compute};
use rand::{prelude::SmallRng, SeedableRng};
use tmp::Tmp;

use crate::common::{generate_plaintext_with_content, generate_random_plaintext_file_with_rng};

mod common;

/// Create file with specified filename and content, write data, compute hash and then
/// remove the file
fn create_plaintext_file_and_hash(content: &str, plaintext_file: impl AsRef<Path>) -> String {
    generate_plaintext_with_content(plaintext_file.as_ref(), content);

    let hasher = Sha256FileHasher::try_new(plaintext_file.as_ref()).unwrap();
    let hash = hasher.start().unwrap();

    remove_file(plaintext_file.as_ref()).unwrap();

    hash.as_hex()
}

#[test]
fn test_small_ascii_file() {
    let tmp = Tmp::new();

    let mut plaintext_file = tmp.path();
    plaintext_file.push("small_file.txt");

    // Empty string hash
    assert_eq!(
        create_plaintext_file_and_hash("", &plaintext_file),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    // Short ascii hash
    assert_eq!(
        create_plaintext_file_and_hash("abc", &plaintext_file),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

fn seeded_hash(rng: &mut SmallRng, length: usize, plaintext_file: impl AsRef<Path>) -> String {
    let plaintext_file = plaintext_file.as_ref();
    generate_random_plaintext_file_with_rng(rng, plaintext_file, length);

    let hasher = Sha256FileHasher::try_new(plaintext_file).unwrap();
    let hash = hasher.start().unwrap();

    remove_file(plaintext_file).unwrap();

    hash.as_hex()
}

#[test]
fn test_big_random_file() {
    let tmp = Tmp::new();

    let mut plaintext_file = tmp.path();
    plaintext_file.push("big_random_file.txt");

    let mut rng = SmallRng::seed_from_u64(0);

    let expected_hashes = vec![
        "a56402dbec9ab2c746d19aa09828a2d3a916241a69f7c27766449ecdd8c5d2f7",
        "f936583147c91e72e75213189927641b813cd9e8d1f5074ca964be4e32386eb7",
        "65021418f2c4a45ce5e31375d8031df0c4eb5170cb38c2db21b678dc811f080e",
        "466ec0036ffe1949cec15a8065edabaec18f48112dacb811227a12c930279149",
        "f136ee17a57272540fc7177ff6b1d57d2b69d4588e13ffeac5a6d6cdfac40512",
        "b977725e7e9ccb22256935172eea0417313be4f4a460a6066a47a5d9266b3d33",
        "210a87f3e88cfa742b38ae57c0cca59a7f418babe6398fd429f0582f5c7c41b2",
        "0d486e51edf2318699e477f0e24de00b4763d380077d07a4eaebecbb169c84a2",
        "bdee7fc2e6aaeee49d5704ec1844ac815b4b68ca662a1f492621ed7da6713937",
        "e5acf1359aaff1267d868d535f0b57e4417322a63a623e6a7fb9fefdc5f2ff39",
        "82c45a3e2c7978bc1d6ecc4e4a1059c976e8c68bec73d4864c54be0a9611e43a",
        "deff7aaf5f68330200bf2f3ce2365e3959cd7c9470e0e422f9fbd671bd7722f2",
        "b89937cd08e6e681a778779bb0cc4baeb4310fee60ea6035d4d718c9d5aed513",
        "97d1e4f4719e342a2144bbe6ea03e4a286569682b2ec3cdd630cdfee7b1cba17",
        "3686bcff75d72d87a7a8ae727a58e4e7d9d7f2f99e9145eef4e1ae02cdc3013c",
        "9b00ff8cc2eae638e1a2a8b0ade864e9a3600923f484959cc6eaf1f86e680a98",
        "bff8522e516cccd91d63fbe021412690b759b11d8e3372e889be5bbae18062c8",
        "03618f442288684de74031b4bfc021d13add5c7a4ad39e34cc4ec2d18bbd15a5",
        "ca246a9f822d912055908a31622ac1218dd21289659b74c20abf4402da920a06",
        "70d20ce2a4c2311cb85c918a28cd7c8ef56f8e2f21c79ece441f17de2539851f",
        "10e0ae86115b69d5f6108ea561ac6691aad22f9fe4ad84fba24bee1dce1b3392",
        "b61eb4490c9bfb460cfae608f2e254e93126aabb6e2c4784941cdef4194098eb",
        "a01b0f9afafe6ca185bcc0e2ac2f5047a09736a121912772448965d8789ba6a3",
        "210f7c7fe119284a7a048bd2218e450790b5dd871b34870171dead3cdf3d72b9",
        "e3093131187eb8fd00c3162af75d2933675cba187aa32ae2b14e304f03c11d52",
        "301eaaf00a1210cd0a600fb5c8e3943e08fcd292a0d3ff6a501adca8cbcf65c7",
        "6cac6a7eab19b6e064b79da3d98250d88ce5e4748a9a0ec27d846d97183ec21d",
        "262baab27d8a6a32cc0934d28d67bfd45d3677a92124b76e95468cae7477905c",
        "be6cad30539a258f759b8f6c6287d7587176a03bf347157b551f10ec3db56c43",
        "7e8debb712efaa22529ec18999bb7a0d3f928385f1d30a1e5ebd093662b451bf",
        "9a6e835f69c6941474f78689bb3c126dc1d4d9a9e5cf00ac71c9c9203abe62b1",
        "747f93865acb9c0d0b0dfcfafc6dbe6a5e2d71710155fd6e552280bb9b16cf41",
        "85b126daaf508275adb5fbb4c10d930a1e57fd0f19a4a494365b16278686bef3",
        "19178622f94db220c91bf2030c67855afc0fa7c4c9cc725023e0fe11b1e55bef",
        "c2303dd3a4c11a19cedb77ba82ad11b5078652b96dc20ad38a9017a44e2f6989",
        "8900507ebaaf7f48f60e545499aa52b601f0d81e5f3b2ab982f6597f18ffc65d",
        "60f978408703a7759af7547ec142813dcf3193b778fa211fea5345da95f0d736",
        "81f759e8310481c1d7f6fd5f78ec64bd655010780cfd929f51d2f1b906657e7d",
        "3a3af6b4cd5d4703475cc717bfef5b9a9c1a8c26952b3ec5c028715f527c8e9c",
        "ff77b2bb53800405cba643f50cb8f557def30d2a42c99511f16c0120d2edef9a",
        "fb19242f57857bda60d3c771e9650c7377c79087fea4edaebc4c973580b56978",
        "898f0ce109146df2ace67590ff3effd649f1f12fe1fa44c4fbb9d07057068dd1",
        "ee61f48764db5e11ae04fc75738d807c604b974b320acfc0d172ddb5136a50f2",
        "0ee7171f588b710561016c050e4e7c20c25ccee4a23f626f12b9adbfc3a5f986",
        "c370e1077fa02b4c8c74f85f94f489080a4c33d25e999c5066f7eafb1c910d23",
        "e1616344314b7524c3c20f9bdc67860fc318013101a7ee53f4f882b26819c4ef",
        "dee99a80d451961a865b2bfb65f9b6a0b7918434bc5bc8b05f934b972f294d8f",
        "d17d430cbb62a99d86e80f16c0f54e808a55acb91cb47f2f68ce1e438490a323",
        "95bbf6ffedf5bca5eebf01004acb30fba1158174669f55bcb7dbdeecd12a99f5",
        "69a05b8aa22b0a0dd28b272723e9a1d64de5a998e207ba6f1235d1e9f90eda41",
        "a31d534321c7ea9e80b7cf99ccbd56230d04aaca0e88c29d358647f420b3dedc",
        "e4866d96b64e93c4917ce756ebcc34d20ff25922161c43942e685835dd6cb291",
        "a70329205708d5afe35da900fea66818847e2992dd9763a28177d7869c62eb72",
        "d5f6852ed8c0ed33b24ad321d2fa74c8831989fd565c361e1c2110a8c464bbe7",
        "4b08bac42fc805ff60b9fc96a98edc583a2f5a8cd37b54d0825d61a64fe71813",
        "b7a44cc36a89c5407e3541fcee944302dd366331aed03fd9a801c61049056d64",
        "a8206c40111d173118cb503a29e3c37963aca76614fb4582a03d43f1470b6068",
        "3ae3b7d59a021f75f405af51cce7bf518cdcc7e3dbfa55afee92b65589051f1e",
        "78cc37e5cdcaa8484e9d7981fdb81a5bf84e397f45e3fc529f3182f52e765511",
        "b6ef493bf5eeb9aca2a6459e13eb532293761a658b1540977ee146dc3af20582",
        "7c8be1213d5a76e87ef7b41ead078cf40df490e0ee824f8186afca1ca88ade1c",
        "41141cc55da488ca3f35ac3b1b14e4e735060d0a5ddcc6debefe566286ff3f26",
        "60eafac782278ca28ce2fe3f00b3157d428e76d0258394538186acd14e66a6cf",
        "9a53d3d2c53a6f304926516d16253159f644b94232054a94f07cea114211ed4a",
        "366ef4c5aa7a95130f525b85f31cc7785022c6a7fa6ae145911f0a1dac03cbc9",
        "13deabc7978a5cb3f8fd489be4531fce2a07a5a5ae89d59b7e90d703b7451a97",
        "c3329d2b8d8f0776409ed5d816a850b157567c7b328da0960670baa0ab168def",
        "e4b559a07974458099ec78046ca8dbc0d4271e9bfeffa6d4f0bfbcf1dd27d595",
        "e07ff305bd1ef415501fcc64de9edaf5d3867011a4ab8029f2dbd03f1a6a114d",
        "7094c09d559550e3d5b4cab9044e73394a59dd950676ea13c708d0b7b0968360",
        "8b3cc6475cc1f7c64bd387a38bb990a234c0fb55e764275c9248d3439090dec3",
        "32caa518b3b3c175962d5ce879f27776326c8e85ae27d1bdfef09c5015c30260",
        "8da8c6c3d2e70f243abdabdf8815c6309a5259ce7fb556acefa4e27dc9543599",
        "08039178ed20612f73fc1e015293e49d4bad92f1745a4bc58082304fdd75d689",
        "36b9a7c89c40c441a1029beea4201d6c1ab32bbfb51548dd963f442776549dbd",
        "0af8b9a7646b4061207941a80cf2108d20e35fb252df3866e3901e5cba297db6",
        "1c32396184933d76506262e8731f73f8fff2e4ce5073734dafebf368c3714f37",
        "dcd5af54536b60d1dcf159de9875ca0f75a76ec040d3d3179b1e025698ab9c09",
        "171b1a89b83c9b78a91c3d60318b589196c3f4730c3896bca89e8ac5121fd7a0",
        "3387a5c78640c3d8a91d364deb2ce1a35d71a5cbd29f49d77707e98fa5d7bf97",
        "fba7e2c49e2c7ae472cae9c6bed498747237c5985c043eed78ff7a3e603da388",
        "51b43b326428b06034371cc57e6c699073fac562c04c623965dbbad229ffac47",
        "b3197c2af0fecc7cc17660c4765131ddad580a45ca50c6b3146d6c0c7bcdd32a",
        "0c75a35c05e65ce416ff358fa41ffe14cc2fce637d765c354499198d001c178f",
        "74acac9ee5a84187662fb9b6e0ef8349b0984ce4dba8958937ff427e83ecf37d",
        "f21c3341555e1f910ecd669e67aa3e33569ee2fb81a2f1157eb1fbee8323002a",
        "7a2a7a4b0b21c0dcec74e5afe0d3ed80fc116fa521b075a619fa3bfa29b84e14",
        "7047f947c69dd7f0a5b3c23d835158753218d62c0dd23838a3c513cde62adb0a",
        "d14739b8a268773ca457b440275f5415e15ad7b7460086fe67059ee3e6ca2381",
        "2c19a2e0af414e3907cf28e432709235769cf95fddc9c391ea8554d3b397c1a5",
        "d8bfcb978b7901c504a8e18435d4980aa1f6b5209a6e90df5c967944584af44a",
        "5cfe63461f7f5bafe95cd84da46421dbe55784c9330230edf56291efe504d271",
        "4137af0ee89bcfed7fe6e4ce08977dec4df4728073733d378b37707d0232e3c6",
        "7d7c1389b563f0a0f495a3f57b549f8142bcc092b86d1b7da30784c7f5c8f84d",
        "795aa6586769c0de311406cb56c5995f6277ecca1ff25a9bdad48e384cd7fe4e",
        "688f6b963a855f8b8ae52468083bd5c85c61f851f268bbf7ddeb16c87463f35a",
        "8358fe39aff00e83bbce2c563925e1303a07d19c85180a76f0610790589e2a93",
        "0e4313cceb0984841bd29f2a8e7cbcc76c141d203d5374363bb35c1d6bc09e5e",
        "88d41df56cf844742f00982c35103063837a1d62aa7b98089fb10d57dde53479",
        "82b6692fb78e3794531b54e986768e3f52cb3a88f389ff7b97763915dad6bb85",
        "62c82801a57179535f53ee0e7c516dc3d70627c70757af8158656417455a1012",
        "4c546cbf6d0125ff6d0e408391ad70ccabb3f7ebb2b123294fa5e1f3ceb61c8b",
        "f871f4649805f70b174f9fee0df96d30f392746cda18b49ed4e510f956627a19",
        "5f36ea2147924eadca42b035001b8536c367bb795477083199852f4b7fac0e1f",
        "259efb3f1b337a005ef1466f5810cb23a63dc8d22df80548a17fed85fe65f8d9",
        "358ebc4edd0e71275828cb38923bcd1897a46e1bc88451fa4f48523d347eda38",
        "8fcac9bb8e0cd6cd016880ff4051775b4b2cfc2bfeb3f5b3991253358f692faa",
        "976cab770d6023ecbaf986daf0563a1fa727538c7b4f12e46d8c30e95d84ddde",
        "6a1ca60e88722de6cee487dd39d2293c816946653b891bb035b0431618c2f4f1",
        "c3801a2078946cf260e84b4322b24a4add609935ac1aaa4877d4d62339036318",
        "23490afee091dbca1ac6760c053d0b7330ef54492fc3b2e3482944c140bb770f",
        "de60975532e8f9d392c3888213a395756801124c6b2cfc6552b2717326d46563",
        "18c804ad12e84df0790c694dbdbdd046f59491c4188f6921a07fbbb1b07f00a2",
        "18c5788af807b4f3b7791eaf47c5bc646296cb0a5afac6aa83291bd54e176238",
        "cb7e7bd5ddd3002fd71ed2233129e71dd01c5f50ae8ebbcf134323229c3afb43",
        "d588bcebbe11c6176b59073f6e3cf88c2af76538c604ca6b9379e69ffd337392",
        "be40c26d8faf7416fb1a2f45ecc17f964d3aabdf765c24db52b49c2f3bd322e7",
        "2e89c2fc2bc7ae4c7df115b087f39d1e1572850813cabfb9a6666103efa7080f",
        "db0a776edfedb512acba49ac4b0f33dbdbc654cf8b7b6f17718bab42df1fb0ab",
        "221dcae0ad83ab06be446cfa8507d0c8224b4bb9b4151b761d49df3b84f82d0f",
        "33273fdba4da75cd1e876d7282c7aa75cf725eaefaf5b1e889c2a1998094f19b",
        "a3f2127b55c98d2b1ca35913e6609f8936cf59df2e38eac32672fd9c28623e5b",
        "4de9920c6e0060c8c62f441522cd2d618d58e10d31d29cd3575e003b56441997",
        "26f17acaaf1e97b0703c55d078f2243b027875fae192d196da91debb1d34d146",
        "3c1d31744641b4162f4a67d28b8524f4264bb545511ead3031113f4be7a7fa4e",
        "b6dea0dbb1f23603e32b2e4cc075acd3a152ba4c94646305df1bc66d8da10488",
        "c99e873190281c02b01e8efd89115d9a4cbce49bf011e6d5b7e6e898663eca58",
        "10ea94452b215226462a2bf44f585d2edc3cb3e72f4e14da16b3730246080042",
    ];

    for i in 0..128 {
        let hash = seeded_hash(&mut rng, 2usize.pow(20), &plaintext_file);
        assert_eq!(hash, expected_hashes[i]);
    }
}
