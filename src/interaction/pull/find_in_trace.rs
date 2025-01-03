use super::Prop;
use crate::prelude::*;

/// Inspired by [`CWeaponPhysCannon::FindObjectTrace`](https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/game/server/hl2/weapon_physcannon.cpp#L2470)
pub(super) fn find_prop_in_trace(
    spatial_query: &SpatialQuery,
    origin: Transform,
    config: &AvianPickupActor,
) -> Option<Prop> {
    // Fun fact: Valve lies to you and actually multiplies this by 4 at this point.
    let test_length = config.interaction_distance;
    let shape_cast_config = ShapeCastConfig::from_max_distance(test_length);
    let hit = spatial_query.cast_ray(
        origin.translation,
        origin.forward(),
        test_length,
        true,
        &config.prop_filter,
    );

    hit.filter(|hit| {
        if let Some(terrain_hit) = spatial_query.cast_ray(
            origin.translation,
            origin.forward(),
            test_length,
            true,
            &config.obstacle_filter,
        ) {
            let occluded = terrain_hit.entity != hit.entity
                && terrain_hit.distance <= hit.distance;
            !occluded
        } else {
            true
        }
    });

    if let Some(hit) = hit {
        Prop {
            entity: hit.entity,
            toi: hit.distance,
        }
        .into()
    } else {
        // This has a half-extent of 4 inches in the 2013 code, which is about 1 cm
        const MAGIC_HALF_EXTENT_ASK_VALVE: f32 = 0.01;
        let fake_aabb_because_parry_cannot_do_aabb_casts =
            Cuboid::from_size(Vec3::splat(2. * MAGIC_HALF_EXTENT_ASK_VALVE)).into();
        let hit = spatial_query.cast_shape(
            &fake_aabb_because_parry_cannot_do_aabb_casts,
            origin.translation,
            origin.rotation,
            origin.forward(),
            &shape_cast_config,
            //test_length,
            //false,
            &config.prop_filter,
        );
        hit.filter(|hit| {
            if let Some(terrain_hit) = spatial_query.cast_shape(
                &fake_aabb_because_parry_cannot_do_aabb_casts,
                origin.translation,
                origin.rotation,
                origin.forward(),
                &shape_cast_config,
                //test_length,
                //false,
                &config.obstacle_filter,
            ) {
                let occluded = terrain_hit.entity != hit.entity
                    && terrain_hit.distance <= hit.distance;
                !occluded
            } else {
                true
            }
        })
        .map(|hit| Prop {
            entity: hit.entity,
            toi: hit.distance,
        })
    }
}
