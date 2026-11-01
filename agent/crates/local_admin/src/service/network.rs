pub async fn probe_server_connection(server_url: &str) -> (bool, u64, String) {
    let trimmed = server_url.trim();
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return (
            false,
            0,
            "URL phải bắt đầu bằng http:// hoặc https://".to_string(),
        );
    }

    let (is_https, rest) = if let Some(stripped) = trimmed.strip_prefix("https://") {
        (true, stripped)
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        (false, stripped)
    } else {
        return (false, 0, "URL không hợp lệ".to_string());
    };

    let host_port_part = rest.split('/').next().unwrap_or("");
    let (host, port) = if let Some((h, p)) = host_port_part.split_once(':') {
        (
            h,
            p.parse::<u16>().unwrap_or(if is_https { 443 } else { 80 }),
        )
    } else {
        (host_port_part, if is_https { 443 } else { 80 })
    };

    if host.is_empty() {
        return (false, 0, "Địa chỉ máy chủ trống".to_string());
    }

    let addr = format!("{}:{}", host, port);
    let start = std::time::Instant::now();
    let connect_res = tokio::time::timeout(
        std::time::Duration::from_millis(3000),
        tokio::net::TcpStream::connect(&addr),
    )
    .await;

    let latency = start.elapsed().as_millis() as u64;

    match connect_res {
        Ok(Ok(_)) => (
            true,
            latency,
            format!("Kết nối thành công tới {} ({}ms)", addr, latency),
        ),
        Ok(Err(e)) => (
            false,
            latency,
            format!("Không thể kết nối tới {}: {}", addr, e),
        ),
        Err(_) => (
            false,
            3000,
            format!("Kết nối tới {} quá thời gian (Timeout 3s)", addr),
        ),
    }
}
