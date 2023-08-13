use std::{collections::BTreeSet, sync::Arc};

use tokio::sync::MutexGuard;

use crate::server::room::RoomInner;

pub fn select(
    room_inner_lock: &mut MutexGuard<'_, RoomInner>,
    user_id: &Arc<str>,
    ids: BTreeSet<usize>,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut accepted_set = BTreeSet::new();
    let mut rejected_set = BTreeSet::new();

    for id in ids {
        if room_inner_lock.figures.contains_key(&id) {
            accepted_set.insert(id);
        } else {
            rejected_set.insert(id);
        }
    }

    let backup = accepted_set.clone();
    if let Some(item) = room_inner_lock.selected_figures.get_mut(user_id) {
        item.append(&mut accepted_set);
    } else {
        room_inner_lock
            .selected_figures
            .insert(user_id.clone(), accepted_set);
    }

    (backup, rejected_set)
}

pub fn unselect(
    room_inner_lock: &mut MutexGuard<'_, RoomInner>,
    user_id: &Arc<str>,
    ids: BTreeSet<usize>,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut accepted_set = BTreeSet::new();
    let mut rejected_set = BTreeSet::new();

    for id in ids {
        if room_inner_lock.figures.contains_key(&id) {
            accepted_set.insert(id);
        } else {
            rejected_set.insert(id);
        }
    }

    if let Some(item) = room_inner_lock.selected_figures.get_mut(user_id) {
        for id in accepted_set.iter() {
            item.remove(id);
        }
        if item.is_empty() {
            room_inner_lock.selected_figures.remove(user_id);
        }
    } else {
        unreachable!()
    }

    (accepted_set, rejected_set)
}
