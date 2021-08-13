pub mod meta;
mod lod;
use std::sync::Arc;

use crate::{
    regionmanager::meta::RegionId,
    job::JobManager,
};
use lod::terrain::TerrainLod;

#[derive(Debug, Clone)]
pub struct Region {
    id: RegionId,
    jobmanager: Arc<JobManager>,

    pub block: TerrainLod,
    temp: TerrainLod,
    light: TerrainLod,
    evil: TerrainLod,
    civ: TerrainLod,
}

impl Region {
    pub fn new(id: RegionId, jobmanager: Arc<JobManager>) -> Self {
        Self {
            id,
            jobmanager,
            block: TerrainLod::default(),
            temp: TerrainLod::default(),
            light: TerrainLod::default(),
            evil: TerrainLod::default(),
            civ: TerrainLod::default(),
        }
    }
}

fn rasterize(region: &Region) -> Vec<u64> {
    let mut res = Vec::new();

    // iterate over all Region9 / chunk5 / Block0 / subBlock that dont have children in RECT XYZ
    //region.block

    res
}



fn plant_trees(region: &Region) -> Vec<u64> {
    let mut res = Vec::new();

    // iterate over all Region9 / chunk5 / Block0 / subBlock that dont have children in RECT XYZ
    // acces blocks around

    res
}



fn corrosion(region: &Region) -> Vec<u64> {
    let mut res = Vec::new();

    // iterate over all Region9 / chunk5 / Block0 / subBlock that dont have children in RECT XYZ
    // access neighbours

    res
}



/*

pub type aaa = LodLayer<e::Terain>;

fn example() {
    let own = e::Terain::new();
    let t8 = own.get(Vec3::new(1,1,1));
    //let tn = own.get2((1,2,3))
}

*/

/*
#[cfg(test)]
mod tests {
    use crate::{
        regionmanager::meta::RegionId,
        job::JobManager,
        lodstore::LodLayer,
        lodstore::Layer,
        region::lod::terrain::Terrain,
        region::Region,
    };
    use vek::*;
    use std::sync::Arc;
    use std::{thread, time};
/*
    #[test]
    fn createRegion() {
        let mut r = Region::new((0,0), Arc::new(JobManager::new()));
        r.block.make_at_least(Vec3::new(0,0,0), Vec3::new(65535,65535,65535), 9);
    }*/

    #[test]
    fn createRegionToBlock() {
        // one region fully blown needs around 80 GB, 1/8 of a region needs 10GB for full block level
        let mut r = Region::new((0,0), Arc::new(JobManager::new()));
        r.block.make_at_least(Vec3::new(0,0,0), Vec3::new(65535/2,65535/2,65535/2), 0);
        r.block.make_at_least(Vec3::new(0,0,0), Vec3::new(65535/2,65535/2,65535/2), 0);

        thread::sleep(time::Duration::from_secs(100));

    }
/*
    #[test]
    fn createRegionToSubBlock() {
        let mut r = Region::new((0,0), Arc::new(JobManager::new()));
        r.block.make_at_least(Vec3::new(0,0,0), Vec3::new(65535,65535,65535), -4);
    }*/
}

*/