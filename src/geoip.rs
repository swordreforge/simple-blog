use maxminddb::geoip2::City;
use maxminddb::Reader;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use once_cell::sync::Lazy;

/// GeoIP 查询结果
#[derive(Debug, Clone, Default)]
pub struct GeoLocation {
    pub country: String,
    pub city: String,
    pub region: String,
}

/// GeoIP 数据库读取器（使用 Lazy 实现单例）
static GEOIP_READER: Lazy<Option<Arc<Reader<Vec<u8>>>>> = Lazy::new(|| {
    // 尝试从多个位置查找 GeoIP 数据库文件
    let db_paths = vec![
        "data/GeoLite2-City.mmdb",
        "myblog-gogogo/data/GeoLite2-City.mmdb",
        "/home/swordreforge/project/rustblog/data/GeoLite2-City.mmdb",
        "/home/swordreforge/project/rustblog/myblog-gogogo/data/GeoLite2-City.mmdb",
    ];

    for db_path in &db_paths {
        if Path::new(db_path).exists() {
            match Reader::open_readfile(db_path) {
                Ok(reader) => {
                    println!("GeoIP 数据库加载成功: {}", db_path);
                    return Some(Arc::new(reader));
                }
                Err(e) => {
                    eprintln!("无法加载 GeoIP 数据库 {}: {}", db_path, e);
                }
            }
        }
    }

    eprintln!("警告: 未找到 GeoIP 数据库文件，地理位置查询将返回 'unknown'");
    None
});

/// 根据 IP 地址查询地理位置信息
pub fn lookup_ip(ip: &str) -> GeoLocation {
    // 如果没有加载 GeoIP 数据库，返回默认值
    let reader: &Reader<Vec<u8>> = match GEOIP_READER.as_ref() {
        Some(r) => r,
        None => return GeoLocation::default(),
    };

    // 解析 IP 地址
    let ip_addr: IpAddr = match ip.parse() {
        Ok(addr) => addr,
        Err(_) => return GeoLocation::default(),
    };

    // 查询 GeoIP 数据库
    match reader.lookup::<City>(ip_addr) {
        Ok(city) => {
            // 获取国家名称
            let country = if let Some(country) = city.country {
                if let Some(names) = country.names {
                    names.get("zh-CN")
                        .or_else(|| names.get("en"))
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            // 获取城市名称
            let city_name = if let Some(city_data) = city.city {
                if let Some(names) = city_data.names {
                    names.get("zh-CN")
                        .or_else(|| names.get("en"))
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            // 获取地区名称
            let region = if let Some(subdivisions) = city.subdivisions {
                if let Some(sub) = subdivisions.into_iter().next() {
                    if let Some(names) = sub.names {
                        names.get("zh-CN")
                            .or_else(|| names.get("en"))
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    } else {
                        "unknown".to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            GeoLocation {
                country,
                city: city_name,
                region,
            }
        }
        Err(e) => {
            eprintln!("GeoIP 查询失败 {}: {}", ip, e);
            GeoLocation::default()
        }
    }
}

/// 获取 GeoIP 数据库是否已加载
pub fn is_database_loaded() -> bool {
    GEOIP_READER.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_google_dns() {
        let geo = lookup_ip("8.8.8.8");
        // Google DNS 应该能查询到美国的位置
        assert!(!geo.country.is_empty());
        println!("8.8.8.8 -> Country: {}, City: {}, Region: {}", geo.country, geo.city, geo.region);
    }

    #[test]
    fn test_lookup_invalid_ip() {
        let geo = lookup_ip("invalid-ip");
        assert_eq!(geo.country, "unknown");
        assert_eq!(geo.city, "unknown");
        assert_eq!(geo.region, "unknown");
    }
}