pub fn drain_filter<T, P: Fn(&T) -> bool>(vec: &mut Vec<T>, pred: P) -> Vec<T> {
    let mut filtered = Vec::new();

    let mut index = 0;
    while index < vec.len() {
        if pred(&vec[index]) {
            filtered.push(vec.remove(index));
        } else {
            index += 1;
        }
    }

    filtered
}
