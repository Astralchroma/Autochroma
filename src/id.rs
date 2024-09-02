use poise::serenity_prelude::GuildId;
use sqlx::{encode::IsNull, error::BoxDynError, Database, Decode, Encode, Type, TypeInfo};
use std::ops::Deref;

#[derive(Clone, Copy)]
pub struct Id(u64);

impl<D: Database> Type<D> for Id
where
	i64: Type<D>,
{
	fn type_info() -> D::TypeInfo {
		<i64 as Type<D>>::type_info()
	}

	fn compatible(ty: &<D>::TypeInfo) -> bool {
		ty.type_compatible(&<i64 as Type<D>>::type_info())
	}
}

impl<'r, D: Database> Decode<'r, D> for Id
where
	i64: Decode<'r, D>,
{
	fn decode(value: <D>::ValueRef<'r>) -> Result<Self, BoxDynError> {
		<i64 as Decode<D>>::decode(value).map(|value| Self(value as u64))
	}
}

impl<'r, D: Database> Encode<'r, D> for Id
where
	i64: Encode<'r, D>,
{
	fn encode_by_ref(&self, buffer: &mut <D>::ArgumentBuffer<'r>) -> Result<IsNull, BoxDynError> {
		<i64 as Encode<D>>::encode_by_ref(&(self.0 as i64), buffer)
	}
}

impl Deref for Id {
	type Target = u64;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl From<GuildId> for Id {
	fn from(value: GuildId) -> Self {
		Self(value.get())
	}
}
