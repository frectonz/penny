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

      {/* Orange glow - top right */}
      <div
        style={{
          position: "absolute",
          top: "-100px",
          right: "-100px",
          width: "500px",
          height: "500px",
          borderRadius: "50%",
          background:
            "radial-gradient(circle, rgba(249,115,22,0.12) 0%, transparent 70%)",
          display: "flex",
        }}
      />

      {/* Orange glow - bottom left */}
      <div
        style={{
          position: "absolute",
          bottom: "-150px",
          left: "-100px",
          width: "400px",
          height: "400px",
          borderRadius: "50%",
          background:
            "radial-gradient(circle, rgba(249,115,22,0.06) 0%, transparent 70%)",
          display: "flex",
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
          alignItems: "flex-start",
          padding: "80px 80px",
          height: "100%",
          position: "relative",
        }}
      >
        {/* Logo + Title */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "24px",
            marginBottom: "16px",
          }}
        >
          <div
            style={{
              width: "80px",
              height: "80px",
              position: "relative",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <svg
              width="80"
              height="80"
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
                fontSize: "32px",
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
              fontSize: "96px",
              fontWeight: 800,
              color: "#fafafa",
              letterSpacing: "-0.05em",
              lineHeight: 1,
              display: "flex",
            }}
          >
            Penny
          </div>
        </div>

        {/* Tagline */}
        <div
          style={{
            fontSize: "36px",
            color: "#f97316",
            fontWeight: 600,
            marginBottom: "24px",
            display: "flex",
          }}
        >
          Serverless for your servers.
        </div>

        {/* Description */}
        <div
          style={{
            fontSize: "22px",
            color: "#737373",
            maxWidth: "700px",
            lineHeight: 1.5,
            display: "flex",
          }}
        >
          A reverse proxy that starts your apps on demand and kills them when
          idle. Ten side projects, one VPS, zero waste.
        </div>

        {/* Bottom terminal hint */}
        <div
          style={{
            position: "absolute",
            bottom: "60px",
            left: "80px",
            display: "flex",
            alignItems: "center",
            gap: "8px",
          }}
        >
          <div
            style={{
              fontSize: "16px",
              color: "#525252",
              fontFamily: "monospace",
              display: "flex",
              alignItems: "center",
              gap: "8px",
            }}
          >
            <span style={{ color: "#f97316" }}>$</span>
            <span>penny serve penny.toml</span>
          </div>
        </div>

        {/* Domain label */}
        <div
          style={{
            position: "absolute",
            bottom: "60px",
            right: "80px",
            display: "flex",
            alignItems: "center",
            gap: "8px",
          }}
        >
          <div
            style={{
              fontSize: "16px",
              color: "#525252",
              display: "flex",
            }}
          >
            pennyproxy.com
          </div>
        </div>
      </div>
    </div>,
    {
      width: 1200,
      height: 630,
    },
  );
}
