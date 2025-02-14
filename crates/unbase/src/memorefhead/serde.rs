use crate::slab::memoref_serde::*;
use crate::util::serde::*;
use super::*;

impl StatefulSerialize for MemoRefHead {
    fn serialize<S>(&self, serializer: S, helper: &SerializeHelper) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for memoref in self.0.iter(){
            seq.serialize_element( &SerializeWrapper( memoref, helper ) )?;
        }
        seq.end()
    }
}

pub struct MemoRefHeadSeed<'a> { pub dest_slab: &'a Slab, pub origin_slabref: &'a SlabRef }

impl<'a> DeserializeSeed for MemoRefHeadSeed<'a> {
    type Value = MemoRefHead;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where D: Deserializer
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'a> Visitor for MemoRefHeadSeed<'a> {
    type Value = MemoRefHead;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
       formatter.write_str("Sequence of MemoRefs")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
       where V: SeqVisitor
    {

        let mut memorefs : Vec<MemoRef> = Vec::new();

        while let Some(memopeer) = visitor.visit_seed( MemoRefSeed{ dest_slab: self.dest_slab, origin_slabref: self.origin_slabref })? {
            memorefs.push(memopeer);
        };

        Ok(MemoRefHead(memorefs))
    }
}
