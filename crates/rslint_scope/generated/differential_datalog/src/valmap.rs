//! Tracks the state of DDlog output relations in memory.  Provides update callback to be invoked when the DB
//! changes; implements methods to dump the entire database or individual relations.
//! Used for testing.

#![allow(non_snake_case, dead_code)]

use std::collections::btree_map::{BTreeMap, Entry};
use std::convert::{AsMut, AsRef};
use std::fmt::Display;
use std::io;

use crate::ddlog::DDlogConvert;
use crate::program::RelId;

/* Stores a set of changes to output tables.
 */
#[derive(Debug, Default)]
pub struct DeltaMap<V> {
    map: BTreeMap<RelId, BTreeMap<V, isize>>,
}

impl<V> AsMut<BTreeMap<RelId, BTreeMap<V, isize>>> for DeltaMap<V> {
    fn as_mut(&mut self) -> &mut BTreeMap<RelId, BTreeMap<V, isize>> {
        &mut self.map
    }
}

impl<V> AsRef<BTreeMap<RelId, BTreeMap<V, isize>>> for DeltaMap<V> {
    fn as_ref(&self) -> &BTreeMap<RelId, BTreeMap<V, isize>> {
        &self.map
    }
}

impl<V> std::ops::Deref for DeltaMap<V> {
    type Target = BTreeMap<RelId, BTreeMap<V, isize>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<V> IntoIterator for DeltaMap<V> {
    type Item = (RelId, BTreeMap<V, isize>);
    type IntoIter = std::collections::btree_map::IntoIter<RelId, BTreeMap<V, isize>>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<V: Display + Ord + Clone> DeltaMap<V> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::default(),
        }
    }

    pub fn singleton(relid: RelId, delta: BTreeMap<V, isize>) -> Self {
        let mut map = BTreeMap::new();
        map.insert(relid, delta);
        Self { map }
    }

    pub fn format<R>(&self, w: &mut dyn io::Write) -> io::Result<()>
    where
        R: DDlogConvert,
    {
        for (relid, relmap) in &self.map {
            w.write_fmt(format_args!("{}:\n", R::relid2name(*relid).unwrap()))?;
            for (val, weight) in relmap {
                w.write_fmt(format_args!("{}: {}\n", *val, *weight))?;
            }
            w.write_fmt(format_args!("\n"))?;
        }
        Ok(())
    }

    pub fn format_rel(&mut self, relid: RelId, w: &mut dyn io::Write) -> io::Result<()> {
        let map = self.get_rel(relid);
        for (val, weight) in map {
            w.write_fmt(format_args!("{}: {}\n", *val, *weight))?;
        }
        Ok(())
    }

    pub fn format_as_sets<R>(&self, w: &mut dyn io::Write) -> io::Result<()>
    where
        R: DDlogConvert,
    {
        for (relid, map) in &self.map {
            w.write_fmt(format_args!("{}:\n", R::relid2name(*relid).unwrap()))?;
            for (val, weight) in map {
                if *weight == 1 {
                    w.write_fmt(format_args!("{}\n", *val))?;
                } else {
                    w.write_fmt(format_args!("{} {:+}\n", *val, *weight))?;
                }
                //assert_eq!(*weight, 1, "val={}, weight={}", *val, *weight);
            }
            w.write_fmt(format_args!("\n"))?;
        }
        Ok(())
    }

    pub fn format_rel_as_set(&mut self, relid: RelId, w: &mut dyn io::Write) -> io::Result<()> {
        let map = self.get_rel(relid);
        for (val, weight) in map {
            if *weight == 1 {
                w.write_fmt(format_args!("{}\n", *val))?;
            } else {
                w.write_fmt(format_args!("{} {:+}\n", *val, *weight))?;
            }
            //assert_eq!(*weight, 1, "val={}, weight={}", *val, *weight);
        }
        Ok(())
    }

    pub fn get_rel(&mut self, relid: RelId) -> &BTreeMap<V, isize> {
        self.map.entry(relid).or_insert_with(BTreeMap::default)
    }

    pub fn try_get_rel(&self, relid: RelId) -> Option<&BTreeMap<V, isize>> {
        self.map.get(&relid)
    }

    pub fn clear_rel(&mut self, relid: RelId) -> BTreeMap<V, isize> {
        self.map.remove(&relid).unwrap_or_else(BTreeMap::default)
    }

    pub fn update(&mut self, relid: RelId, x: &V, diff: isize) {
        //println!("set_update({}) {:?} {}", rel, *x, insert);
        let entry = self
            .map
            .entry(relid)
            .or_insert_with(BTreeMap::default)
            .entry((*x).clone());
        match entry {
            Entry::Vacant(vacant) => {
                vacant.insert(diff);
            }
            Entry::Occupied(mut occupied) => {
                if *occupied.get() == -diff {
                    occupied.remove();
                } else {
                    *occupied.get_mut() += diff;
                }
            }
        };
    }
}
