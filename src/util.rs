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

#[cfg(not(test))]
static UNIQUE_IDENT_COUNTER: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
thread_local! {
    static UNIQUE_IDENT_COUNTER: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

#[cfg(not(test))]
pub(crate) fn unique_ident() -> syn::Ident {
    let n = UNIQUE_IDENT_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    quote::format_ident!("__cain_ident__{}", n)
}

#[cfg(test)]
pub(crate) fn in_test<F: FnOnce() -> R, R>(f: F) -> R {
    UNIQUE_IDENT_COUNTER.with(|counter| {
        counter.set(0);
    });
    f()
}

#[cfg(test)]
pub(crate) fn unique_ident() -> syn::Ident {
    let n = UNIQUE_IDENT_COUNTER.with(|counter| {
        let n = counter.get();
        counter.set(n + 1);
        n
    });

    quote::format_ident!("__cain_ident__{}", n)
}
