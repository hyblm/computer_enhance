pub const EARTH_RADIUS: f64 = 6372.8;

pub fn reference_haversine(x0: f64, y0: f64, x1: f64, y1: f64, radius: f64) -> f64 {
    let lat1 = y0;
    let lat2 = y1;
    let lon1 = x0;
    let lon2 = x1;

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = (d_lat / 2.).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.).sin().powi(2);
    let c = 2. * a.sqrt().asin();

    radius * c
}
