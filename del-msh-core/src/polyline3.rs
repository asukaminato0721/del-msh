//! methods related to 3D polyline

use num_traits::AsPrimitive;

/// the center of gravity
pub fn cg<T>(vtx2xyz: &[T]) -> nalgebra::Vector3<T>
where
    T: nalgebra::RealField + Copy,
    f64: AsPrimitive<T>,
{
    let num_vtx = vtx2xyz.len() / 3;
    let mut cg = nalgebra::Vector3::<T>::zeros();
    let mut w = T::zero();
    for iseg in 0..num_vtx - 1 {
        let ip0 = iseg;
        let ip1 = iseg + 1;
        let p0 = crate::vtx2xyz::to_navec3(vtx2xyz, ip0);
        let p1 = crate::vtx2xyz::to_navec3(vtx2xyz, ip1);
        let len = (p0 - p1).norm();
        cg += (p0 + p1).scale(0.5f64.as_() * len);
        w += len;
    }
    cg / w
}

/// bi-normal vector on each vertex
pub fn vtx2framex<T>(vtx2xyz: &[T]) -> nalgebra::Matrix3xX<T>
where
    T: nalgebra::RealField + 'static + Copy,
    f64: AsPrimitive<T>,
{
    use crate::vtx2xyz::to_navec3;
    let num_vtx = vtx2xyz.len() / 3;
    let mut vtx2bin = nalgebra::Matrix3xX::<T>::zeros(num_vtx);
    {
        // first segment
        let v01 = (to_navec3(vtx2xyz, 1) - to_navec3(vtx2xyz, 0)).into_owned();
        let (x, _) = del_geo_nalgebra::vec3::frame_from_z_vector(v01);
        vtx2bin.column_mut(0).copy_from(&x);
    }
    for iseg1 in 1..num_vtx - 1 {
        let iv0 = iseg1 - 1;
        let iv1 = iseg1;
        let iv2 = iseg1 + 1;
        let iseg0 = iseg1 - 1;
        let v01 = to_navec3(vtx2xyz, iv1) - to_navec3(vtx2xyz, iv0);
        let v12 = to_navec3(vtx2xyz, iv2) - to_navec3(vtx2xyz, iv1);
        let rot = del_geo_nalgebra::mat3::minimum_rotation_matrix(v01, v12);
        let b01: nalgebra::Vector3<T> = vtx2bin.column(iseg0).into_owned();
        let b12: nalgebra::Vector3<T> = rot * b01;
        vtx2bin.column_mut(iseg1).copy_from(&b12);
    }
    {
        let a: nalgebra::Vector3<T> = vtx2bin.column(num_vtx - 2).into();
        vtx2bin.column_mut(num_vtx - 1).copy_from(&a);
    }
    vtx2bin
}

pub fn framez<T>(vtx2xyz: &[T], i_vtx: usize) -> nalgebra::Vector3<T>
where
    T: nalgebra::RealField + Copy,
{
    let num_vtx = vtx2xyz.len() / 3;
    assert!(i_vtx < num_vtx);
    if i_vtx == 0 {
        let p1 = crate::vtx2xyz::to_navec3(vtx2xyz, 0);
        let p2 = crate::vtx2xyz::to_navec3(vtx2xyz, 1);
        return (p2 - p1).normalize();
    }
    if i_vtx == num_vtx - 1 {
        let p0 = crate::vtx2xyz::to_navec3(vtx2xyz, num_vtx - 2);
        let p1 = crate::vtx2xyz::to_navec3(vtx2xyz, num_vtx - 1);
        return (p1 - p0).normalize();
    }
    let p0 = crate::vtx2xyz::to_navec3(vtx2xyz, i_vtx - 1);
    let p1 = crate::vtx2xyz::to_navec3(vtx2xyz, i_vtx);
    let p2 = crate::vtx2xyz::to_navec3(vtx2xyz, i_vtx + 1);
    ((p1 - p0).normalize() + (p2 - p1).normalize()).normalize()
}

pub fn vtx2framey<T>(vtx2xyz: &[T], vtx2framex: &nalgebra::Matrix3xX<T>) -> nalgebra::Matrix3xX<T>
where
    T: nalgebra::RealField + 'static + Copy,
    f64: AsPrimitive<T>,
{
    let num_vtx = vtx2xyz.len() / 3;
    assert_eq!(vtx2framex.ncols(), num_vtx);
    let mut vtx2framey = nalgebra::Matrix3xX::<T>::zeros(num_vtx);
    for i_vtx in 0..num_vtx {
        let framez = framez(vtx2xyz, i_vtx);
        let framex = vtx2framex.column(i_vtx);
        vtx2framey
            .column_mut(i_vtx)
            .copy_from(&framez.cross(&framex));
    }
    vtx2framey
}

pub fn normal_binormal<T>(vtx2xyz: &[T]) -> (nalgebra::Matrix3xX<T>, nalgebra::Matrix3xX<T>)
where
    T: nalgebra::RealField + Copy,
{
    let num_vtx = vtx2xyz.len() / 3;
    let mut vtx2bin = nalgebra::Matrix3xX::<T>::zeros(num_vtx);
    let mut vtx2nrm = nalgebra::Matrix3xX::<T>::zeros(num_vtx);
    for ivtx1 in 1..num_vtx - 1 {
        let ivtx0 = (ivtx1 + num_vtx - 1) % num_vtx;
        let ivtx2 = (ivtx1 + 1) % num_vtx;
        let v0 = crate::vtx2xyz::to_navec3(vtx2xyz, ivtx0);
        let v1 = crate::vtx2xyz::to_navec3(vtx2xyz, ivtx1);
        let v2 = crate::vtx2xyz::to_navec3(vtx2xyz, ivtx2);
        let v01 = v1 - v0;
        let v12 = v2 - v1;
        let binormal = v12.cross(&v01);
        vtx2bin.column_mut(ivtx1).copy_from(&binormal.normalize());
        let norm = (v01 + v12).cross(&binormal);
        vtx2nrm.column_mut(ivtx1).copy_from(&norm.normalize());
    }
    {
        let c1 = vtx2nrm.column(1).into_owned();
        vtx2nrm.column_mut(0).copy_from(&c1);
    }
    {
        let c1 = vtx2nrm.column(num_vtx - 2).into_owned();
        vtx2nrm.column_mut(num_vtx - 1).copy_from(&c1);
    }
    {
        let c1 = vtx2bin.column(1).into_owned();
        vtx2bin.column_mut(0).copy_from(&c1);
    }
    {
        let c1 = vtx2bin.column(num_vtx - 2).into_owned();
        vtx2bin.column_mut(num_vtx - 1).copy_from(&c1);
    }
    (vtx2nrm, vtx2bin)
}

#[allow(clippy::identity_op)]
pub fn to_trimesh3_capsule<T>(
    vtxl2xyz: &[T],
    ndiv_circum: usize,
    ndiv_longtitude: usize,
    r: T,
) -> (Vec<usize>, Vec<T>)
where
    T: nalgebra::RealField + Copy + num_traits::Float + num_traits::FloatConst,
    f64: AsPrimitive<T>,
    f32: AsPrimitive<T>,
    usize: AsPrimitive<T>,
{
    assert!(ndiv_circum > 2);
    let num_vtxl = vtxl2xyz.len() / 3;
    let vtxl2framex = vtx2framex(vtxl2xyz);
    let vtxl2framey = vtx2framey(vtxl2xyz, &vtxl2framex);
    //
    let ndiv_length = vtxl2xyz.len() / 3 - 1;
    let (tri2vtx, vtx2xyz) = crate::trimesh3_primitive::cylinder_closed_end_yup::<T>(
        T::one(),
        T::one(),
        ndiv_circum,
        2 * ndiv_longtitude + ndiv_length - 2,
        true,
    );
    let tri2vtx = Vec::<usize>::from(tri2vtx.as_slice());
    let mut vtx2xyz = Vec::<T>::from(vtx2xyz.as_slice());
    assert_eq!(
        vtx2xyz.len() / 3,
        (2 * ndiv_longtitude + ndiv_length - 1) * ndiv_circum + 2
    );
    let pi: T = (std::f32::consts::PI).as_();
    let half: T = 0.5.as_();
    {
        // south pole
        let p0 = crate::vtx2xyz::to_navec3(vtxl2xyz, 0);
        let ez = framez(vtxl2xyz, 0);
        let q = p0 - ez * r;
        vtx2xyz[0] = q.x;
        vtx2xyz[1] = q.y;
        vtx2xyz[2] = q.z;
    }
    for ir in 0..ndiv_longtitude {
        let p0 = crate::vtx2xyz::to_navec3(vtxl2xyz, 0);
        let ex = vtxl2framex.column(0);
        let ey = vtxl2framey.column(0);
        let ez = framez(vtxl2xyz, 0);
        let t0 = pi * half * (ndiv_longtitude - 1 - ir).as_() / ndiv_longtitude.as_();
        let c0 = r * num_traits::Float::cos(t0);
        for ic in 0..ndiv_circum {
            let theta = 2.as_() * pi * ic.as_() / ndiv_circum.as_();
            let q = p0
                + ez.scale(-r * num_traits::Float::sin(t0))
                + ey.scale(c0 * num_traits::Float::cos(theta))
                + ex.scale(c0 * num_traits::Float::sin(theta));
            vtx2xyz[(1 + ir * ndiv_circum + ic) * 3 + 0] = q.x;
            vtx2xyz[(1 + ir * ndiv_circum + ic) * 3 + 1] = q.y;
            vtx2xyz[(1 + ir * ndiv_circum + ic) * 3 + 2] = q.z;
        }
    }
    for il in 0..ndiv_length - 1 {
        let p0 = crate::vtx2xyz::to_navec3(vtxl2xyz, il + 1);
        let ex = vtxl2framex.column(il + 1);
        let ey = vtxl2framey.column(il + 1);
        for ic in 0..ndiv_circum {
            let theta = 2.as_() * pi * ic.as_() / ndiv_circum.as_();
            let q = p0
                + ey.scale(r * num_traits::Float::cos(theta))
                + ex.scale(r * num_traits::Float::sin(theta));
            vtx2xyz[(1 + (il + ndiv_longtitude) * ndiv_circum + ic) * 3 + 0] = q.x;
            vtx2xyz[(1 + (il + ndiv_longtitude) * ndiv_circum + ic) * 3 + 1] = q.y;
            vtx2xyz[(1 + (il + ndiv_longtitude) * ndiv_circum + ic) * 3 + 2] = q.z;
        }
    }
    for ir in 0..ndiv_longtitude {
        let p0 = crate::vtx2xyz::to_navec3(vtxl2xyz, num_vtxl - 1);
        let ex = vtxl2framex.column(num_vtxl - 1);
        let ey = vtxl2framey.column(num_vtxl - 1);
        let ez = framez(vtxl2xyz, num_vtxl - 1);
        let t0 = pi * half * ir.as_() / ndiv_longtitude.as_();
        let c0 = r * num_traits::Float::cos(t0);
        for ic in 0..ndiv_circum {
            let theta = 2.as_() * pi * ic.as_() / ndiv_circum.as_();
            let q = p0
                + ez.scale(r * num_traits::Float::sin(t0))
                + ey.scale(c0 * num_traits::Float::cos(theta))
                + ex.scale(c0 * num_traits::Float::sin(theta));
            vtx2xyz[(1 + (ir + ndiv_length + ndiv_longtitude - 1) * ndiv_circum + ic) * 3 + 0] =
                q.x;
            vtx2xyz[(1 + (ir + ndiv_length + ndiv_longtitude - 1) * ndiv_circum + ic) * 3 + 1] =
                q.y;
            vtx2xyz[(1 + (ir + ndiv_length + ndiv_longtitude - 1) * ndiv_circum + ic) * 3 + 2] =
                q.z;
        }
    }
    {
        // North Pole
        let p0 = crate::vtx2xyz::to_navec3(vtxl2xyz, num_vtxl - 1);
        let ez = framez(vtxl2xyz, num_vtxl - 1);
        let q = p0 + ez * r;
        let np = vtx2xyz.len() / 3;
        vtx2xyz[(np - 1) * 3 + 0] = q.x;
        vtx2xyz[(np - 1) * 3 + 1] = q.y;
        vtx2xyz[(np - 1) * 3 + 2] = q.z;
    }
    (tri2vtx, vtx2xyz)
}

/*
pub fn nearest_to_polyline3<T>(
    poly_a: &Vec::<nalgebra::Vector3::<T>>,
    poly_b: &Vec::<nalgebra::Vector3::<T>>) -> (T,T,T)
    where T: nalgebra::RealField + Copy,
          f64: AsPrimitive<T>,
          usize: AsPrimitive<T>
{
    let mut res: (T,T,T) = (T::max_value().unwrap(), T::zero(), T::zero());
    for ia in 0..poly_a.len() -1 {
        let a0 = &poly_a[ia];
        let a1 = &poly_a[ia+1];
        for ib in 0..poly_b.len() -1 {
            let b0 = &poly_b[ib];
            let b1 = &poly_b[ib+1];
            let dis = del_geo::edge3::nearest_to_edge3(a0,a1,b0,b1);
            if dis.0 < res.0 {
                res.0 = dis.0;
                res.1 = ia.as_() + dis.1;
                res.2 = ib.as_() + dis.2;
            }
        }
    }
    res
}
*/

pub fn contacting_pair(poly2vtx: &[usize], vtx2xyz: &[f32], dist0: f32) -> (Vec<usize>, Vec<f32>) {
    let num_poly = poly2vtx.len() - 1;
    let mut pair_idx = Vec::<usize>::new();
    let mut pair_prm = Vec::<f32>::new();
    for i_poly in 0..num_poly {
        for j_poly in i_poly + 1..num_poly {
            for i_seg in poly2vtx[i_poly]..poly2vtx[i_poly + 1] - 1 {
                let pi = crate::vtx2xyz::to_navec3(vtx2xyz, i_seg);
                let qi = crate::vtx2xyz::to_navec3(vtx2xyz, i_seg + 1);
                for j_seg in poly2vtx[j_poly]..poly2vtx[j_poly + 1] - 1 {
                    let pj = crate::vtx2xyz::to_navec3(vtx2xyz, j_seg);
                    let qj = crate::vtx2xyz::to_navec3(vtx2xyz, j_seg + 1);
                    let (dist, ri, rj) =
                        del_geo_nalgebra::edge3::nearest_to_edge3(&pi, &qi, &pj, &qj);
                    if dist > dist0 {
                        continue;
                    }
                    pair_idx.extend([i_poly, j_poly]);
                    pair_prm.push((i_seg - poly2vtx[i_poly]) as f32 + ri);
                    pair_prm.push((j_seg - poly2vtx[j_poly]) as f32 + rj);
                }
            }
        }
    }
    (pair_idx, pair_prm)
}

pub fn position_from_barycentric_coordinate<T>(vtx2xyz: &[T], r: T) -> nalgebra::Vector3<T>
where
    T: num_traits::Float + nalgebra::RealField + AsPrimitive<usize>,
    usize: AsPrimitive<T>,
{
    let ied: usize = r.as_();
    let ned = vtx2xyz.len() / 3 - 1;
    dbg!(r, ied, ned);
    assert!(ied < ned);
    let p0 = crate::vtx2xyz::to_navec3(vtx2xyz, ied);
    let p1 = crate::vtx2xyz::to_navec3(vtx2xyz, ied + 1);
    let r0 = r - ied.as_();
    p0 + (p1 - p0).scale(r0)
}

pub fn smooth<T>(vtx2xyz: &[T], r: T, num_iter: usize) -> Vec<T>
where
    T: nalgebra::RealField + Copy,
    f64: AsPrimitive<T>,
{
    let num_vtx = vtx2xyz.len() / 3;
    let mut vtx2xyz1 = Vec::from(vtx2xyz);
    for _iter in 0..num_iter {
        for ip1 in 1..num_vtx - 1 {
            let ip0 = (ip1 + num_vtx - 1) % num_vtx;
            let ip2 = (ip1 + 1) % num_vtx;
            let p0 = crate::vtx2xyz::to_navec3(&vtx2xyz1, ip0);
            let p1 = crate::vtx2xyz::to_navec3(&vtx2xyz1, ip1);
            let p2 = crate::vtx2xyz::to_navec3(&vtx2xyz1, ip2);
            let p1n = (p0 + p2).scale(0.5f64.as_() * r) + p1.scale(T::one() - r);
            vtx2xyz1[ip1 * 3] = p1n.x;
            vtx2xyz1[ip1 * 3 + 1] = p1n.y;
            vtx2xyz1[ip1 * 3 + 2] = p1n.z;
        }
    }
    vtx2xyz1
}

pub fn length<T>(vtx2xyz: &[T]) -> T
where
    T: num_traits::Float + std::ops::AddAssign + std::fmt::Debug,
{
    assert_eq!(vtx2xyz.len() % 3, 0);
    let num_vtx = vtx2xyz.len() / 3;
    let mut len = T::zero();
    for i_vtx in 0..num_vtx - 1 {
        let j_vtx = i_vtx + 1;
        let elen = del_geo_core::edge3::length(
            arrayref::array_ref!(vtx2xyz, i_vtx * 3, 3),
            arrayref::array_ref!(vtx2xyz, j_vtx * 3, 3),
        );
        len += elen;
    }
    len
}
fn reduce_recursive(
    vtx2flg: &mut [i32],
    i_vtx0: usize,
    i_vtx1: usize,
    vtx2xyz: &[f32],
    threshold: f32,
) {
    assert!(i_vtx1 as i64 - i_vtx0 as i64 > 1);
    let p0 = arrayref::array_ref![vtx2xyz, i_vtx0 * 3, 3];
    let p1 = arrayref::array_ref![vtx2xyz, i_vtx1 * 3, 3];
    let vtx_farest = {
        let mut vtx_nearest = (usize::MAX, 0.);
        for i_vtxm in i_vtx0 + 1..i_vtx1 {
            let pm = arrayref::array_ref![vtx2xyz, i_vtxm * 3, 3];
            let pn = del_geo_core::edge3::nearest_to_point3(p0, p1, pm);
            let dist = del_geo_core::edge3::length(pm, &pn);
            if dist >= vtx_nearest.1 {
                vtx_nearest = (i_vtxm, dist);
            }
        }
        vtx_nearest
    };
    assert_ne!(vtx_farest.0, usize::MAX);
    assert!(i_vtx0 < vtx_farest.0);
    assert!(vtx_farest.0 < i_vtx1);
    if vtx_farest.1 > threshold {
        vtx2flg[vtx_farest.0] = 1; // this is fixed
    }
    if vtx_farest.0 - i_vtx0 > 1 {
        reduce_recursive(vtx2flg, i_vtx0, vtx_farest.0, vtx2xyz, threshold);
    }
    if i_vtx1 - vtx_farest.0 > 1 {
        reduce_recursive(vtx2flg, vtx_farest.0, i_vtx1, vtx2xyz, threshold);
    }
}

/// <https://en.wikipedia.org/wiki/Ramer%E2%80%93Douglas%E2%80%93Peucker_algorithm>
pub fn reduce(vtx2xyz: &[f32], threshold: f32) -> Vec<f32> {
    let num_vtx = vtx2xyz.len() / 3;
    let mut vtx2flg = vec![0; num_vtx]; // 0: free, 1: fix
    vtx2flg[0] = 1;
    vtx2flg[num_vtx - 1] = 1;
    reduce_recursive(&mut vtx2flg, 0, num_vtx - 1, vtx2xyz, threshold);
    // dbg!(&vtx2flg);
    let vtx2xyz_reduced: Vec<f32> = vtx2xyz
        .chunks(3)
        .enumerate()
        .filter(|&(iv, _xyz)| vtx2flg[iv] == 1)
        .flat_map(|(_iv, xyz)| [xyz[0], xyz[1], xyz[2]])
        .collect();
    vtx2xyz_reduced
}

#[test]
fn test_reduce() -> anyhow::Result<()> {
    let vtx2xy = crate::polyloop2::from_circle(1.0, 100);
    let vtx2xyz = crate::vtx2xy::to_vtx2xyz(&vtx2xy);
    let vtx2xyz_reduced = reduce(&vtx2xyz, 0.01);
    crate::io_obj::save_vtx2xyz_as_polyloop("../target/reduce_polyline.obj", &vtx2xyz_reduced, 3)?;
    Ok(())
}
