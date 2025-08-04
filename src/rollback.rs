// TODO: Does not really work, double ref on 1st and 4th.
// Try to make to work...
pub fn do_or_rollback_<'a, T, K: PartialEq + Clone, R>(
    to_modify: &mut T,
    get_conflict_keys: impl Fn(&T) -> K,
    to_do: impl FnOnce(&mut T) -> R,
    on_conflict: impl FnOnce(K)
) -> R {
    let before = get_conflict_keys(to_modify);
    let to_return = to_do(to_modify);
    let after = get_conflict_keys(to_modify);
    
    if before != after {
        on_conflict(before.clone())
    } 

    to_return
}
