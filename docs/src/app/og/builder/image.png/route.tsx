import { ImageResponse } from "next/og";

export const dynamic = "force-static";

export function GET() {
  return new ImageResponse(
    <div
      style={{
        width: "100%",
        height: "100%",
        display: "flex",
        flexDirection: "column",
        backgroundColor: "#0a0a0a",
        position: "relative",
        overflow: "hidden",
      }}
    >
      {/* Grid pattern */}
      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          backgroundImage:
            "radial-gradient(circle, rgba(255,255,255,0.03) 1px, transparent 1px)",
          backgroundSize: "32px 32px",
        }}
      />

      {/* Orange glow */}
      <div
        style={{
          position: "absolute",
          top: "50%",
          right: "-50px",
          width: "400px",
          height: "400px",
          borderRadius: "50%",
          background:
            "radial-gradient(circle, rgba(249,115,22,0.1) 0%, transparent 70%)",
          display: "flex",
          transform: "translateY(-50%)",
        }}
      />

      {/* Top border accent */}
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          height: "3px",
          background:
            "linear-gradient(to right, transparent, #f97316, transparent)",
          display: "flex",
        }}
      />

      {/* Main content */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          justifyContent: "center",
          padding: "80px",
          height: "100%",
          position: "relative",
        }}
      >
        {/* Logo + site name */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "12px",
            marginBottom: "48px",
          }}
        >
          <div
            style={{
              width: "36px",
              height: "36px",
              position: "relative",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <svg
              width="36"
              height="36"
              viewBox="0 0 100 100"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
              style={{ position: "absolute" }}
            >
              <title>Penny logo</title>
              <circle
                cx="50"
                cy="50"
                r="45"
                stroke="#f97316"
                strokeWidth="6"
                fill="none"
              />
              <circle
                cx="50"
                cy="50"
                r="35"
                stroke="#f97316"
                strokeWidth="3"
                fill="none"
              />
            </svg>
            <span
              style={{
                fontSize: "15px",
                fontWeight: 700,
                color: "#f97316",
                position: "relative",
              }}
            >
              P
            </span>
          </div>
          <div
            style={{
              fontSize: "20px",
              color: "#737373",
              display: "flex",
            }}
          >
            Penny
          </div>
        </div>

        {/* Title */}
        <div
          style={{
            display: "flex",
            alignItems: "baseline",
            marginBottom: "16px",
          }}
        >
          <span
            style={{
              fontSize: "72px",
              fontWeight: 800,
              color: "#fafafa",
              letterSpacing: "-0.04em",
              lineHeight: 1,
            }}
          >
            penny
          </span>
          <span
            style={{
              fontSize: "72px",
              fontWeight: 800,
              color: "#f97316",
              letterSpacing: "-0.04em",
              lineHeight: 1,
            }}
          >
            .
          </span>
          <span
            style={{
              fontSize: "72px",
              fontWeight: 800,
              color: "#fafafa",
              letterSpacing: "-0.04em",
              lineHeight: 1,
            }}
          >
            toml
          </span>
        </div>

        {/* Subtitle */}
        <div
          style={{
            fontSize: "28px",
            color: "#737373",
            display: "flex",
            marginBottom: "40px",
          }}
        >
          Interactive Configuration Builder
        </div>

        {/* Faux TOML snippet */}
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: "4px",
            padding: "24px",
            border: "1px solid #1a1a1a",
            backgroundColor: "#0f0f0f",
            maxWidth: "500px",
          }}
        >
          <div style={{ display: "flex", gap: "0px", fontSize: "16px" }}>
            <span style={{ color: "#525252" }}>[</span>
            <span style={{ color: "#f97316" }}>"myapp.example.com"</span>
            <span style={{ color: "#525252" }}>]</span>
          </div>
          <div style={{ display: "flex", gap: "0px", fontSize: "16px" }}>
            <span style={{ color: "#999" }}>address</span>
            <span style={{ color: "#525252" }}> = </span>
            <span style={{ color: "#22c55e" }}>"127.0.0.1:3001"</span>
          </div>
          <div style={{ display: "flex", gap: "0px", fontSize: "16px" }}>
            <span style={{ color: "#999" }}>command</span>
            <span style={{ color: "#525252" }}> = </span>
            <span style={{ color: "#22c55e" }}>"node server.js"</span>
          </div>
        </div>

        {/* Domain label */}
        <div
          style={{
            position: "absolute",
            bottom: "60px",
            right: "80px",
            display: "flex",
            fontSize: "16px",
            color: "#525252",
          }}
        >
          pennyproxy.com
        </div>
      </div>
    </div>,
    {
      width: 1200,
      height: 630,
    },
  );
}
