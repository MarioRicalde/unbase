/* Memo
 * A memo is an immutable message.
*/
pub mod serde;

use std::collections::HashMap;
use std::{fmt};
use std::sync::Arc;

use subject::{SubjectId};
use slab::MemoRef;
use memorefhead::*;
use network::{SlabRef,SlabPresence};
use super::*;

//pub type MemoId = [u8; 32];
pub type MemoId = u64;

// All portions of this struct should be immutable

#[derive(Clone)]
pub struct Memo {
    pub id: u64,
    pub owning_slab_id: SlabId,
    pub subject_id: Option<SubjectId>,
    pub inner: Arc<MemoInner>
}
pub struct MemoInner {
    pub id: u64,
    pub subject_id: Option<SubjectId>,
    pub parents: MemoRefHead,
    pub body: MemoBody
}

#[derive(Debug)]
pub enum MemoBody{
    SlabPresence{ p: SlabPresence, r: Option<MemoRefHead> }, // TODO: split out root_index_seed conveyance to another memobody type
    Relation(HashMap<RelationSlotId,(SubjectId,MemoRefHead)>),
    Edit(HashMap<String, String>),
    FullyMaterialized     { v: HashMap<String, String>, r: RelationSlotSubjectHead },
    PartiallyMaterialized { v: HashMap<String, String>, r: RelationSlotSubjectHead },
    Peering(MemoId,Option<SubjectId>,MemoPeerList),
    MemoRequest(Vec<MemoId>,SlabRef)
}

type RelationSlotSubjectHead = HashMap<RelationSlotId,(SubjectId,MemoRefHead)>;

#[derive(Debug)]
pub struct MemoPeerList (pub Vec<MemoPeer>);

#[derive(Debug)]
pub struct MemoPeer {
    pub slabref: SlabRef,
    pub status: MemoPeeringStatus
}

#[derive(Debug,Clone,PartialEq,Serialize,Deserialize)]
pub enum MemoPeeringStatus{
    Resident,
    Participating,
    NonParticipating,
    Unknown
}


/*
use std::hash::{Hash, Hasher};

impl Hash for MemoId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.originSlab.hash(state);
        self.id.hash(state);
    }
}
*/

impl fmt::Debug for Memo{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let inner = &self.inner;
        fmt.debug_struct("Memo")
           .field("id", &inner.id)
           .field("subject_id", &inner.subject_id)
           .field("parents", &inner.parents)
           .field("body", &inner.body)
           .finish()
    }
}

impl Memo {
    pub fn get_parent_head (&self) -> MemoRefHead {
        self.inner.parents.clone()
    }
    pub fn get_values (&self) -> Option<(HashMap<String, String>,bool)> {

        match self.inner.body {
            MemoBody::Edit(ref v)
                => Some((v.clone(),false)),
            MemoBody::FullyMaterialized { ref v, r: _ }
                => Some((v.clone(),true)),
            _   => None
        }
    }
    pub fn get_relations (&self) -> Option<(HashMap<RelationSlotId, (SubjectId, MemoRefHead)>,bool)> {

        match self.inner.body {
            MemoBody::Relation(ref r)
                => Some((r.clone(),false)),
            MemoBody::FullyMaterialized { v: _, ref r }
                => Some((r.clone(),true)),
            _   => None
        }
    }
    pub fn does_peering (&self) -> bool {
        match self.inner.body {
            MemoBody::MemoRequest(_,_) => {
                false
            }
            MemoBody::Peering(_,_,_) => {
                false
            }
            MemoBody::SlabPresence{p:_, r:_} => {
                false
            }
            _ => {
                true
            }
        }
    }
    pub fn descends (&self, memoref: &MemoRef, slab: &Slab) -> bool {
        //TODO: parallelize this
        //TODO: Use sparse-vector/beacon to avoid having to trace out the whole lineage
        //      Should be able to stop traversal once happens-before=true. Cannot descend a thing that happens after


        // breadth-first
        for parent in self.inner.parents.iter() {
            if parent == memoref {
                return true
            };
        }

        // Ok now depth
        for parent in self.inner.parents.iter() {
            if parent.descends(&memoref,slab) {
                return true
            }
        }
        return false;
    }
}