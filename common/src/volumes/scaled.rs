use crate::vol::{BaseVol, ReadVol, SizedVol, Vox};
use vek::*;

pub struct Scaled<V> {
    pub inner: V,
    pub scale: Vec3<f32>,
}

impl<V: BaseVol> BaseVol for Scaled<V> {
    type Error = V::Error;
    type Vox = V::Vox;
}

impl<V: ReadVol> ReadVol for Scaled<V>
where
    V::Vox: Vox,
{
    #[inline(always)]
    fn get(&self, pos: Vec3<i32>) -> Result<&Self::Vox, Self::Error> {
        // let ideal_pos = pos.map2(self.scale, |e, scale| (e as f32 + 0.5) / scale);
        // let pos = ideal_pos.map(|e| e.trunc() as i32);
        let min_pos = pos.map2(self.scale, |e, scale| ((e as f32) / scale).floor() as i32);
        let max_pos = pos.map2(self.scale, |e, scale| {
            ((e as f32 + 1.0) / scale).ceil() as i32
        });
        let pos = pos.map2(self.scale, |e, scale| {
            (((e as f32 + 0.5) / scale) - 0.5).round() as i32
        });

        // let ideal_search_size = Vec3::<f32>::one() / self.scale;
        let range_iter = |i: usize| {
            std::iter::successors(Some(0), |p| Some(if *p < 0 { -*p } else { -(*p + 1) }))
                .take_while(move |p| {
                    (min_pos[i]..max_pos[i])
                    /* ((ideal_pos[i] - ideal_search_size[i] / 2.0).ceil() as i32
                        ..(ideal_pos[i] + ideal_search_size[i] / 2.0).ceil() as i32) */
                        .contains(&(pos[i] + *p))
                })
        };
        range_iter(0)
            .flat_map(|i| {
                range_iter(1).map(move |j| range_iter(2).map(move |k| Vec3::new(i, j, k)))
            })
            .flatten()
            .map(|offs| self.inner.get(pos + offs))
            .find(|vox| vox.as_ref().map(|v| !v.is_empty()).unwrap_or(false))
            .unwrap_or_else(|| self.inner.get(pos))
    }
}

impl<V: SizedVol> SizedVol for Scaled<V> {
    #[inline(always)]
    fn lower_bound(&self) -> Vec3<i32> {
        self.inner
            .lower_bound()
            .map2(self.scale, |e, scale| (e as f32 * scale).floor() as i32)
    }

    #[inline(always)]
    fn upper_bound(&self) -> Vec3<i32> {
        self.inner
            .upper_bound()
            .map2(self.scale, |e, scale| (e as f32 * scale).ceil() as i32)
    }
}
