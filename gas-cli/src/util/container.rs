pub fn vec_remove_indices<T>(source: &mut Vec<T>, indices: &[usize]) -> Vec<T> {
    let mut removed: Vec<T> = Vec::new();

    let original = std::mem::take(source);

    for (index, item) in original.into_iter().enumerate() {
        if indices.contains(&index) {
            removed.push(item);
            continue;
        }

        source.push(item);
    }

    removed
}
