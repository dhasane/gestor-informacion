

/// Convierte los archivos encontrados en DIR en un json
pub fn files_as_json(dir: String) -> String {
    let vec: Vec<String> = get_files_in_dir(dir);
    let json = serde_json::to_string(&vec);

    match json {
        Ok(it) => it,
        Err(_) => "".to_string(),
    }
}

/// Consigue los archivos que se encuentran en DIR y retorna una lista
/// de strings, los cuales representan a los archivos en este
/// directorio
pub fn get_files_in_dir(dir: String) -> Vec<String> {
    let paths: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|r| -> String {
            if let Ok(a) = r {
                a.file_name().into_string().unwrap()
            } else {
                "".to_string()
            }
        })
        .collect();

    paths
}
