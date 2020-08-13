use crate::models::{ComponentId, Timestamp};
use std::cmp::max;
use std::collections::HashMap;

/// Dependency Vector Manager
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Clone)]
struct DependencyVector {
    map: HashMap<ComponentId, Timestamp>,
    id: ComponentId,
}

impl DependencyVector {
    #[allow(dead_code)]
    pub fn new(self_id: ComponentId, components: Vec<ComponentId>) -> DependencyVector {
        let mut map: HashMap<ComponentId, Timestamp> = HashMap::new();
        map.insert(self_id.clone(), 0);
        for c in components {
            map.insert(c, 0);
        }
        DependencyVector {
            id: self_id,
            map: map,
        }
    }

    #[allow(dead_code)]
    pub fn set_self_ts(&mut self, ts: Timestamp) -> Result<(), ()> {
        if ts < self.map[&self.id] {
            return Err(());
        }
        self.map.insert(self.id.clone(), ts);
        return Ok(());
    }

    #[allow(dead_code)]
    pub fn update(&mut self, map: &HashMap<ComponentId, Timestamp>) -> Result<(), ()> {
        let mut new_vals: HashMap<ComponentId, Timestamp> = HashMap::new();

        // check if rollback dependency is inconsistent
        if let Some(ts) = map.get(&self.id) {
            if *ts > self.map[&self.id] {
                return Err(());
            }
        }

        for (id, self_ts) in self.map.iter() {
            if *id != self.id {
                if let Some(other_ts) = map.get(&id) {
                    new_vals.insert(id.clone(), max(*self_ts, *other_ts));
                }
            }
        }

        for (id, ts) in new_vals {
            self.map.insert(id, ts);
        }

        return Ok(());
    }

    #[allow(dead_code)]
    pub fn get_map(&self) -> &HashMap<ComponentId, Timestamp> {
        &self.map
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::models::*;

    fn get_id(x: u32) -> ComponentId {
        ComponentId {
            federate_id: x,
            federation_id: x,
        }
    }

    #[test]
    fn new_creates_correct_dependency_vector() {
        let self_id = get_id(1);
        let components = vec![get_id(1), get_id(2), get_id(3)];
        let manager = DependencyVector::new(self_id, components);
        let mut map: HashMap<ComponentId, Timestamp> = HashMap::new();
        map.insert(get_id(1), 0);
        map.insert(get_id(2), 0);
        map.insert(get_id(3), 0);
        assert_eq!(
            manager,
            DependencyVector {
                id: get_id(1),
                map: map
            }
        );
    }

    #[test]
    fn setselfts_returns_err_if_new_ts_is_lower_than_current_ts() {
        let mut manager = DependencyVector::new(get_id(1), vec![get_id(1), get_id(2)]);
        manager.set_self_ts(10).unwrap();
        match manager.set_self_ts(5) {
            Err(()) => (),
            Ok(()) => panic!(),
        }
    }

    #[test]
    fn setselfts_updates_self_ts_correctly() {
        let mut manager = DependencyVector::new(get_id(1), vec![get_id(1), get_id(2)]);
        let mut clone = manager.clone();
        manager.set_self_ts(10).unwrap();
        assert_ne!(manager, clone);
        clone.map.insert(get_id(1), 10);
        assert_eq!(manager, clone);
    }

    /// Consider that:
    ///     self_dvec is the local dependency vector;
    ///     other_dvec is the received dependency vector;
    ///     self_id is the id of the local component.
    ///
    /// The rollback dependency is inconsistent if other_dvec[self_id] > self_dvec[self_id]
    #[test]
    fn update_returns_err_if_rollback_dependency_is_inconsistent() {
        let mut manager = DependencyVector::new(get_id(1), vec![get_id(1), get_id(2)]);
        let mut map: HashMap<ComponentId, Timestamp> = HashMap::new();
        map.insert(get_id(1), 10);
        map.insert(get_id(2), 0);
        match manager.update(&map) {
            Err(()) => (),
            Ok(()) => panic!(),
        }
    }

    // #[test]
    // fn update_returns_err_if_received_dvec_has_different_keys() {
    //     let mut man1 = DependencyVector::new(get_id(1), vec![get_id(1), get_id(2)]);
    //     let mut man2 = man1.clone();

    //     let mut map: HashMap<ComponentId, Timestamp> = HashMap::new();
    //     map.insert(get_id(1), 0);
    //     match man1.update(&map) {
    //         Err(()) => (),
    //         Ok(()) => panic!(),
    //     }

    //     map.clear();
    //     map.insert(get_id(1), 0);
    //     map.insert(get_id(2), 0);
    //     map.insert(get_id(3), 0);
    //     match man2.update(&map) {
    //         Err(()) => (),
    //         Ok(()) => panic!(),
    //     }
    // }

    #[test]
    fn update_changes_values_correctly() {
        let mut manager = DependencyVector::new(get_id(1), vec![get_id(1), get_id(2), get_id(3)]);
        manager.set_self_ts(10).unwrap();
        let mut map: HashMap<ComponentId, Timestamp> = HashMap::new();
        map.insert(get_id(1), 0);
        map.insert(get_id(2), 10);
        map.insert(get_id(3), 20);
        manager.update(&map).unwrap();
        map.insert(get_id(1), 10);
        assert_eq!(manager.map, map);
        assert_eq!(manager.id, get_id(1));

        map.clear();
        map.insert(get_id(1), 8);
        map.insert(get_id(2), 15);
        map.insert(get_id(3), 16);
        manager.update(&map).unwrap();
        map.insert(get_id(1), 10);
        map.insert(get_id(3), 20);

        println!("manager.map -> {:#?}", manager.map);
        println!("map -> {:#?}", map);

        assert_eq!(manager.map, map);
        assert_eq!(manager.id, get_id(1));
    }
}
