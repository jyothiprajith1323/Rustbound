//! Linear physical custody tokens representing industrial materials.

use core::marker::PhantomData;

/// Marker trait implemented by all valid physical resource types.
pub trait ResourceKind {
    fn name() -> &'static str;
}

/// Raw magnetic iron ore extracted from Aethelgard subterranean seams.
pub struct IronOre;
impl ResourceKind for IronOre {
    fn name() -> &'static str {
        "Iron Ore"
    }
}

/// Native copper chunks used in power grid and thermal components.
pub struct CopperOre;
impl ResourceKind for CopperOre {
    fn name() -> &'static str {
        "Copper Ore"
    }
}

/// Structural steel or iron ingots produced by smelters.
pub struct RefinedIngot;
impl ResourceKind for RefinedIngot {
    fn name() -> &'static str {
        "Refined Ingot"
    }
}

/// Quality grade of an extracted mineral chunk.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OreGrade {
    LowGrade,
    Standard,
    HighPurity,
}

/// An owned physical resource chunk.
///
/// # Physical Custody & Linear Typing
///
/// This struct represents an authentic physical object on Aethelgard.
/// It deliberately does **NOT** implement [`Clone`] or [`Copy`].
///
/// Moving this struct transfers physical custody. If you load an item onto a
/// conveyor or deposit it into storage, this value is moved. Attempting to use
/// it afterward triggers Rust compiler error `E0382: use of moved value`.
#[must_use = "resource chunks represent physical items; dropping them wastes material"]
pub struct ResourceChunk<T: ResourceKind> {
    id: u64,
    grade: OreGrade,
    _marker: PhantomData<T>,
}

impl<T: ResourceKind> ResourceChunk<T> {
    /// Internal constructor used exclusively by machine drivers.
    #[inline]
    pub(crate) const fn new(id: u64, grade: OreGrade) -> Self {
        Self {
            id,
            grade,
            _marker: PhantomData,
        }
    }

    /// Returns the unique serial ID of this physical item token.
    #[inline]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the metallurgical grade of this resource.
    #[inline]
    pub const fn grade(&self) -> OreGrade {
        self.grade
    }

    /// Returns the human-readable material name.
    #[inline]
    pub fn material_name(&self) -> &'static str {
        T::name()
    }
}
