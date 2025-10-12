// use bytes::{BufMut, BytesMut};
// use postgres_types::{FromSql, IsNull, ToSql, Type, to_sql_checked};
// use std::{
//     error::Error,
//     io::{Cursor, Read},
// };
// use tank_core::Interval;

// #[derive(Debug)]
// pub(crate) struct IntervalWrap(pub(crate) Interval);

// impl ToSql for IntervalWrap {
//     fn to_sql(&self, ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
//     where
//         Self: Sized,
//     {
//         if !matches!(*ty, Type::INTERVAL) {
//             return Err(tank_core::Error::msg(format!(
//                 "Cannot write Interval sql into type: {}",
//                 ty
//             ))
//             .into());
//         }
//         out.put_i64(self.0.months);
//         out.put_i64(self.0.days);
//         out.put_i128(self.0.nanos);
//         Ok(IsNull::No)
//     }

//     fn accepts(ty: &Type) -> bool
//     where
//         Self: Sized,
//     {
//         matches!(*ty, Type::INTERVAL)
//     }

//     to_sql_checked!();
// }

// impl<'a> FromSql<'a> for IntervalWrap {
//     fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
//         if !matches!(*ty, Type::INTERVAL) {
//             return Err(tank_core::Error::msg(format!(
//                 "Cannot read Interval sql from type: {}",
//                 ty
//             ))
//             .into());
//         }
//         let mut raw = Cursor::new(raw);
//         raw.read_exact(buf)
//     }

//     fn accepts(ty: &Type) -> bool {
//         matches!(*ty, Type::INTERVAL)
//     }
// }
