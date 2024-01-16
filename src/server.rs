use embedded_svc::{http::Headers, ipv4::IpInfo};

use {
    esp_idf_svc::http::{server::EspHttpServer, Method},
    log::info,
};

pub fn start(ip_info: IpInfo) -> anyhow::Result<EspHttpServer<'static>> {
    let ip = ip_info.ip;
    let location = std::format!("https://cdaringe.github.io/gimbal-motion?ip={ip}");

    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: 10240,
        ..Default::default()
    };

    let mut server = EspHttpServer::new(&server_configuration)?;

    server.fn_handler("/", Method::Get, move |req| {
        let conn = req.connection().unwrap_or("unknown");
        info!("handling req from connection: {conn}");
        let mut response = req.into_response(301, None, &[("Location", &location)])?;
        response.flush()?;
        Ok(())
    })?;

    server.fn_handler("/api/gcode", Method::Get, move |req| {
        let conn = req.connection().unwrap_or("unknown");
        let gcode = url::Url::parse(req.uri())?
            .query_pairs()
            .find(|(k, _)| k == "gcode")
            .map(|(_, gcode)| gcode.to_string());
        let (code, message, payload) = match gcode {
            Some(g) => {
                info!("handling req from connection: {conn}");
                (200, "ok", format!("{{ \"ok\": \"{g}\" }}"))
            }
            None => (
                400,
                "bad param",
                format!("{{ \"error\": \"bad gcode param\" }}"),
            ),
        };
        let mut response =
            req.into_response(code, Some(message), &[("content-type", "application/json")])?;
        response.write(payload.as_bytes())?;
        response.flush()?;
        Ok(())
    })?;

    Ok(server)
}
