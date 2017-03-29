use super::*;
use network::slabref::serde::*;
use util::serde::*;

impl<'a> StatefulSerialize for &'a MemoRef {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let shared = &self.shared.lock().unwrap();
        let mut seq = serializer.serialize_seq(Some(4))?;
        seq.serialize_element(&self.id)?;
        seq.serialize_element(&self.subject_id)?;
        match &shared.ptr {
            &MemoRefPtr::Remote      => seq.serialize_element(&false)?,
            &MemoRefPtr::Resident(_) => seq.serialize_element(&true)?,
        };
        seq.serialize_element( &SerializeWrapper(&shared.peers, helper) )?;
        seq.end()
    }
}
impl StatefulSerialize for MemoPeer {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&SerializeWrapper(&self.slabref, helper))?;
        seq.serialize_element(&self.status)?;
        seq.end()
    }
}

pub struct MemoRefSeed<'a> { pub net: &'a Network }

impl<'a> DeserializeSeed for MemoRefSeed<'a> {
    type Value = MemoRef;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MemoRefSeed<'a> {
    type Value = MemoRef;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct MemoRef")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<MemoRef, V::Error>
       where V: SeqVisitor
    {
        let memo_id: MemoId = match visitor.visit()? {
            Some(value) => value,
            None => {
                return Err(DeError::invalid_length(0, &self));
            }
        };
        let subject_id: SubjectId = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(1, &self));
           }
        };
        let has_memo: bool = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(2, &self));
           }
        };

        let peers: Vec<MemoPeer> = match visitor.visit_seed( VecSeed( MemoPeerSeed{ net: self.net } ) )? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(3, &self));
           }
        };

       let memoref = MemoRef {
           id: memo_id,
           subject_id: Some(subject_id),
           shared: Arc::new(Mutex::new(
               MemoRefShared {
                   peers: peers,
                   ptr: match has_memo {
                       true  => MemoRefPtr::Remote,
                       false => MemoRefPtr::Remote
                   }
               }
           ))
       };

       Ok(memoref)
    }
}

#[derive(Clone)]
pub struct MemoPeerSeed<'a> { net: &'a Network }

impl<'a> DeserializeSeed for MemoPeerSeed<'a> {
    type Value = MemoPeer;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MemoPeerSeed<'a> {
    type Value = MemoPeer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("struct MemoPeer")
    }
    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {
        let slabref: SlabRef = match visitor.visit_seed( SlabRefSeed{ net: self.net })? {
            Some(value) => value,
            None => {
                return Err(DeError::invalid_length(0, &self));
            }
        };
        let status: PeeringStatus = match visitor.visit()? {
           Some(value) => value,
           None => {
               return Err(DeError::invalid_length(1, &self));
           }
        };

       Ok(MemoPeer{
           slabref: slabref,
           status: status
       })
    }
}
