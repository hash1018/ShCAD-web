use std::collections::BTreeSet;

use tokio::sync::MutexGuard;

use crate::server::room::RoomInner;

pub fn delete(
    room_inner_lock: &mut MutexGuard<'_, RoomInner>,
    ids: BTreeSet<usize>,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut accepted_set = BTreeSet::new();
    let mut rejected_set = BTreeSet::new();

    let mut remove_vec = Vec::new();
    for (remove_id, set) in room_inner_lock.selected_figures.iter_mut() {
        for i in ids.iter() {
            set.remove(i);
        }
        if set.is_empty() {
            remove_vec.push(remove_id.clone());
        }
    }

    for remove_id in remove_vec {
        room_inner_lock.selected_figures.remove(&remove_id);
    }

    for id in ids.iter() {
        if room_inner_lock.figures.remove(id).is_some() {
            accepted_set.insert(*id);
        } else {
            rejected_set.insert(*id);
        }
    }

    (accepted_set, rejected_set)
}
