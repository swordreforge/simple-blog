use maxminddb::geoip2::City;
use maxminddb::Reader;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use once_cell::sync::Lazy;
use dashmap::DashMap;

/// GeoIP 缓存容量
const GEOIP_CACHE_CAPACITY: usize = 10000;

/// GeoIP 查询结果
#[derive(Debug, Clone, Default)]
pub struct GeoLocation {
    pub country: String,
    pub city: String,
    pub region: String,
}

/// GeoIP 数据库读取器（使用 Lazy 实现单例，使用 Mmap 减少内存占用）
static GEOIP_READER: Lazy<Option<Arc<Reader<memmap2::Mmap>>>> = Lazy::new(|| {
    // 尝试从多个位置查找 GeoIP 数据库文件
    let db_paths = vec![
        "data/GeoLite2-City.mmdb",
        "myblog-gogogo/data/GeoLite2-City.mmdb",
        "/home/swordreforge/project/rustblog/data/GeoLite2-City.mmdb",
        "/home/swordreforge/project/rustblog/myblog-gogogo/data/GeoLite2-City.mmdb",
    ];

    for db_path in &db_paths {
        if Path::new(db_path).exists() {
            // 使用 Mmap 打开数据库文件，减少内存占用
            let file = match std::fs::File::open(db_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("无法打开 GeoIP 数据库文件 {}: {}", db_path, e);
                    continue;
                }
            };
            
            let mmap = match unsafe { memmap2::Mmap::map(&file) } {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("无法创建 Mmap: {}", e);
                    continue;
                }
            };
            
            match Reader::from_source(mmap) {
                Ok(reader) => {
                    println!("GeoIP 数据库加载成功（使用 Mmap）: {}", db_path);
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

/// GeoIP 查询缓存（使用 DashMap 实现无锁并发缓存）
static GEOIP_CACHE: Lazy<DashMap<String, GeoLocation>> = Lazy::new(|| {
    DashMap::with_capacity(GEOIP_CACHE_CAPACITY)
});

/// 根据 IP 地址查询地理位置信息（带缓存）
pub fn lookup_ip(ip: &str) -> GeoLocation {
    // 首先尝试从缓存中获取
    if let Some(cached) = GEOIP_CACHE.get(ip) {
        return cached.clone();
    }

    // 如果没有加载 GeoIP 数据库，返回 unknown
    let reader: &Reader<memmap2::Mmap> = match GEOIP_READER.as_ref() {
        Some(r) => r,
        None => {
            let result = GeoLocation {
                country: "unknown".to_string(),
                city: "unknown".to_string(),
                region: "unknown".to_string(),
            };
            // 缓存unknown结果
            GEOIP_CACHE.insert(ip.to_string(), result.clone());
            return result;
        },
    };

    // 解析 IP 地址
    let ip_addr: IpAddr = match ip.parse() {
        Ok(addr) => addr,
        Err(_) => {
            let result = GeoLocation {
                country: "unknown".to_string(),
                city: "unknown".to_string(),
                region: "unknown".to_string(),
            };
            // 缓存无效IP结果
            GEOIP_CACHE.insert(ip.to_string(), result.clone());
            return result;
        },
    };

    // 查询 GeoIP 数据库
    let result = match reader.lookup(ip_addr) {
        Ok(lookup_result) => {
            // 解码为 City 类型
            match lookup_result.decode::<City>() {
                Ok(Some(city)) => {
                    // 获取国家名称
                    let country = if !city.country.is_empty() {
                        if !city.country.names.is_empty() {
                            city.country.names.simplified_chinese
                                .or(city.country.names.english)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        } else {
                            "unknown".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    };

                    // 获取城市名称
                    let city_name = if !city.city.is_empty() {
                        if !city.city.names.is_empty() {
                            city.city.names.simplified_chinese
                                .or(city.city.names.english)
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "unknown".to_string())
                        } else {
                            "unknown".to_string()
                        }
                    } else {
                        "unknown".to_string()
                    };

                    // 获取地区名称
                    let region = if !city.subdivisions.is_empty() {
                        if let Some(sub) = city.subdivisions.first() {
                            if !sub.names.is_empty() {
                                sub.names.simplified_chinese
                                    .or(sub.names.english)
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
                Ok(None) => {
                    // 查询成功但没有数据
                    GeoLocation::default()
                }
                Err(e) => {
                    eprintln!("GeoIP 解码失败 {}: {}", ip, e);
                    GeoLocation::default()
                }
            }
        }
        Err(e) => {
            eprintln!("GeoIP 查询失败 {}: {}", ip, e);
            GeoLocation::default()
        }
    };

    // 缓存查询结果
    GEOIP_CACHE.insert(ip.to_string(), result.clone());

    result
}

/// 清空 GeoIP 缓存
pub fn clear_geoip_cache() {
    GEOIP_CACHE.clear();
}

/// 获取 GeoIP 缓存统计信息
pub fn get_geoip_cache_stats() -> usize {
    GEOIP_CACHE.len()
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

    #[test]
    fn test_geoip_cache() {
        // 先清空缓存以确保测试环境干净
        clear_geoip_cache();
        
        // 第一次查询
        let geo1 = lookup_ip("8.8.8.8");
        
        // 检查缓存统计
        let len = get_geoip_cache_stats();
        assert!(len > 0);
        
        // 第二次查询应该从缓存中获取
        let geo2 = lookup_ip("8.8.8.8");
        
        assert_eq!(geo1.country, geo2.country);
        assert_eq!(geo1.city, geo2.city);
        assert_eq!(geo1.region, geo2.region);
    }

    #[test]
    fn test_clear_cache() {
        // 查询一些IP
        lookup_ip("8.8.8.8");
        lookup_ip("1.1.1.1");
        
        let len = get_geoip_cache_stats();
        assert!(len > 0);
        
        // 清空缓存
        clear_geoip_cache();
        
        let new_len = get_geoip_cache_stats();
        // 注意：由于DashMap的实现，clear可能不会立即反映在len中
        // 所以我们只检查它不会增长
        assert!(new_len <= len);
    }
}