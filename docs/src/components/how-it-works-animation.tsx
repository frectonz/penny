"use client";

import { useTheme } from "next-themes";
import { useEffect, useState } from "react";

export function HowItWorksAnimation() {
  const { resolvedTheme } = useTheme();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const isDark = !mounted || resolvedTheme !== "light";

  const c = isDark
    ? {
        bg: "#0a0a0a",
        box: "#0f0f0f",
        sec: "#1a1a1a",
        border: "#262626",
        label: "#a1a1a1",
        inactive: "#3a3a3a",
        grid: "rgba(255,255,255,0.03)",
      }
    : {
        bg: "#fafafa",
        box: "#f5f5f5",
        sec: "#e5e5e5",
        border: "#d4d4d4",
        label: "#737373",
        inactive: "#c0c0c0",
        grid: "rgba(0,0,0,0.04)",
      };

  const o = "#f97316";
  const g = "#22c55e";
  const r = "#dc2626";

  return (
    <div className="mx-auto w-full max-w-4xl">
      <svg
        viewBox="0 0 800 300"
        className="w-full h-auto"
        style={{ fontFamily: "'JetBrains Mono', ui-monospace, monospace" }}
        aria-label="How penny works: request arrives, penny starts app, traffic is proxied, idle app is killed"
      >
        <title>How penny works</title>
        <defs>
          <pattern
            id="grid"
            width="20"
            height="20"
            patternUnits="userSpaceOnUse"
          >
            <path
              d="M20 0L0 0 0 20"
              fill="none"
              stroke={c.grid}
              strokeWidth="0.5"
            />
          </pattern>
        </defs>

        {/* Background */}
        <rect width="800" height="300" fill="url(#grid)" rx="8" />

        {/* === BROWSER BOX === */}
        <g>
          <rect
            x="40"
            y="80"
            width="140"
            height="100"
            rx="6"
            fill={c.box}
            stroke={c.border}
            strokeWidth="1"
          />
          <rect x="40" y="80" width="140" height="18" rx="6" fill={c.sec} />
          <rect x="40" y="94" width="140" height="4" fill={c.sec} />
          <circle cx="52" cy="89" r="3" fill={r} />
          <circle cx="62" cy="89" r="3" fill={o} />
          <circle cx="72" cy="89" r="3" fill={g} />
          <rect x="54" y="106" width="100" height="5" rx="2" fill={c.sec} />
          <rect x="54" y="116" width="80" height="5" rx="2" fill={c.sec} />
          <rect x="54" y="126" width="90" height="5" rx="2" fill={c.sec} />
          <rect x="54" y="136" width="70" height="5" rx="2" fill={c.sec} />
          <text
            x="110"
            y="200"
            textAnchor="middle"
            fill={c.label}
            fontSize="10"
          >
            BROWSER
          </text>
        </g>

        {/* === CONNECTION LINE: Browser → Penny === */}
        <line
          x1="180"
          y1="130"
          x2="290"
          y2="130"
          stroke={c.border}
          strokeWidth="1"
        />
        {/* Request packet */}
        <circle cx="0" cy="0" r="4" fill={o} opacity="0">
          <animateMotion
            dur="8s"
            repeatCount="indefinite"
            keyPoints="0;0;0.5;0.5;0.5;0.5;0.5;0.5"
            keyTimes="0;0;0.125;0.15;0.15;1;1;1"
            calcMode="linear"
          >
            <mpath href="#path-browser-penny" />
          </animateMotion>
          <animate
            attributeName="opacity"
            values="0;0;1;0;0;0;0;0"
            keyTimes="0;0.01;0.02;0.14;0.14;1;1;1"
            dur="8s"
            repeatCount="indefinite"
          />
        </circle>
        <path
          id="path-browser-penny"
          d="M180,130 L290,130"
          fill="none"
          stroke="none"
        />

        {/* Request label */}
        <text
          x="235"
          y="122"
          textAnchor="middle"
          fill={o}
          fontSize="7"
          opacity="0"
        >
          REQUEST
          <animate
            attributeName="opacity"
            values="0;0;1;1;0;0;0;0"
            keyTimes="0;0.01;0.05;0.15;0.2;1;1;1"
            dur="8s"
            repeatCount="indefinite"
          />
        </text>

        {/* === PENNY BOX === */}
        <g>
          <rect
            x="295"
            y="70"
            width="210"
            height="140"
            rx="6"
            fill={c.box}
            stroke={o}
            strokeWidth="1.5"
          >
            <animate
              attributeName="stroke-opacity"
              values="1;1;1;0.5;1;1;1;1"
              keyTimes="0;0.12;0.15;0.2;0.25;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </rect>
          <rect x="295" y="70" width="210" height="22" rx="6" fill={c.sec} />
          <rect x="295" y="86" width="210" height="6" fill={c.sec} />
          <text
            x="400"
            y="85"
            textAnchor="middle"
            fill={o}
            fontSize="9"
            fontWeight="600"
            letterSpacing="0.08em"
          >
            PENNY PROXY
          </text>

          {/* Status: routing (default) */}
          <text x="400" y="120" textAnchor="middle" fill={c.label} fontSize="8">
            <tspan>
              Routing traffic
              <animate
                attributeName="visibility"
                values="visible;visible;hidden;hidden"
                keyTimes="0;0.05;0.06;1"
                dur="8s"
                repeatCount="indefinite"
              />
            </tspan>
          </text>

          {/* Status: waking */}
          <text
            x="400"
            y="120"
            textAnchor="middle"
            fill={o}
            fontSize="8"
            opacity="0"
          >
            Waking server...
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0;0;0"
              keyTimes="0;0.12;0.13;0.3;0.31;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>

          {/* Status: health check */}
          <text
            x="400"
            y="120"
            textAnchor="middle"
            fill={g}
            fontSize="8"
            opacity="0"
          >
            {"Health check \u2713"}
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0;0;0"
              keyTimes="0;0.31;0.32;0.43;0.44;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>

          {/* Status: proxying */}
          <text
            x="400"
            y="120"
            textAnchor="middle"
            fill={g}
            fontSize="8"
            opacity="0"
          >
            Proxying traffic
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0;0;0"
              keyTimes="0;0.44;0.45;0.625;0.63;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>

          {/* Status: idle timeout */}
          <text
            x="400"
            y="120"
            textAnchor="middle"
            fill={r}
            fontSize="8"
            opacity="0"
          >
            Idle timeout reached
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0;0;0"
              keyTimes="0;0.75;0.76;0.87;0.88;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>

          {/* Penny logo */}
          <g transform="translate(375, 140)">
            <circle
              cx="25"
              cy="20"
              r="18"
              stroke={o}
              strokeWidth="2"
              fill="none"
              opacity="0.3"
            />
            <circle
              cx="25"
              cy="20"
              r="14"
              stroke={o}
              strokeWidth="1"
              fill="none"
              opacity="0.3"
            />
            <text
              x="25"
              y="20"
              textAnchor="middle"
              dominantBaseline="central"
              fill={o}
              fontSize="14"
              fontWeight="bold"
              fontFamily="system-ui, sans-serif"
              opacity="0.3"
            >
              P
            </text>
          </g>
        </g>

        {/* === CONNECTION LINE: Penny → Server === */}
        <line
          x1="505"
          y1="130"
          x2="615"
          y2="130"
          stroke={c.border}
          strokeWidth="1"
        />
        <path
          id="path-penny-server"
          d="M505,130 L615,130"
          fill="none"
          stroke="none"
        />
        <path
          id="path-server-penny"
          d="M615,130 L505,130"
          fill="none"
          stroke="none"
        />

        {/* Proxy label */}
        <text
          x="560"
          y="122"
          textAnchor="middle"
          fill={g}
          fontSize="7"
          opacity="0"
        >
          PROXY
          <animate
            attributeName="opacity"
            values="0;0;1;1;0;0;0;0"
            keyTimes="0;0.44;0.45;0.625;0.63;1;1;1"
            dur="8s"
            repeatCount="indefinite"
          />
        </text>

        {/* Traffic packets (forward) */}
        {[0, 0.04, 0.08].map((delay) => (
          <g key={`fwd-${delay}`}>
            <circle cx="0" cy="0" r="3" fill={g} opacity="0">
              <animateMotion
                dur="8s"
                repeatCount="indefinite"
                keyPoints="0;0;0;1;1;1;1;1"
                keyTimes={`0;${0.44 + delay};${0.45 + delay};${0.5 + delay};${0.51 + delay};1;1;1`}
                calcMode="linear"
              >
                <mpath href="#path-penny-server" />
              </animateMotion>
              <animate
                attributeName="opacity"
                values="0;0;1;1;0;0"
                keyTimes={`0;${0.44 + delay};${0.45 + delay};${0.5 + delay};${0.51 + delay};1`}
                dur="8s"
                repeatCount="indefinite"
              />
            </circle>
          </g>
        ))}

        {/* Response packets (reverse) */}
        {[0.02, 0.06, 0.1].map((delay) => (
          <g key={`rev-${delay}`}>
            <circle cx="0" cy="0" r="3" fill={o} opacity="0">
              <animateMotion
                dur="8s"
                repeatCount="indefinite"
                keyPoints="0;0;0;1;1;1;1;1"
                keyTimes={`0;${0.5 + delay};${0.51 + delay};${0.56 + delay};${0.57 + delay};1;1;1`}
                calcMode="linear"
              >
                <mpath href="#path-server-penny" />
              </animateMotion>
              <animate
                attributeName="opacity"
                values="0;0;1;1;0;0"
                keyTimes={`0;${0.5 + delay};${0.51 + delay};${0.56 + delay};${0.57 + delay};1`}
                dur="8s"
                repeatCount="indefinite"
              />
            </circle>
          </g>
        ))}

        {/* === SERVER BOX === */}
        <g>
          <rect
            x="620"
            y="80"
            width="140"
            height="100"
            rx="6"
            fill={c.box}
            stroke={c.border}
            strokeWidth="1"
            strokeDasharray="4 3"
          >
            <animate
              attributeName="stroke"
              values={`${c.border};${c.border};${o};${g};${g};${r};${c.border};${c.border}`}
              keyTimes="0;0.12;0.15;0.35;0.625;0.75;0.87;1"
              dur="8s"
              repeatCount="indefinite"
            />
            <animate
              attributeName="stroke-dasharray"
              values="4 3;4 3;none;none;none;none;4 3;4 3"
              keyTimes="0;0.12;0.15;0.35;0.625;0.75;0.87;1"
              dur="8s"
              repeatCount="indefinite"
            />
            <animate
              attributeName="stroke-width"
              values="1;1;1.5;1.5;1.5;1.5;1;1"
              keyTimes="0;0.12;0.15;0.35;0.625;0.75;0.87;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </rect>

          {/* Status indicator dot */}
          <circle cx="636" cy="100" r="4" fill={c.inactive}>
            <animate
              attributeName="fill"
              values={`${c.inactive};${c.inactive};${o};${g};${g};${r};${c.inactive};${c.inactive}`}
              keyTimes="0;0.12;0.15;0.35;0.625;0.75;0.87;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </circle>

          {/* Server hostname */}
          <text x="646" y="104" fill={c.label} fontSize="9" fontWeight="600">
            myapp.example.com
            <animate
              attributeName="fill"
              values={`${c.label};${c.label};${o};${g};${g};${r};${c.label};${c.label}`}
              keyTimes="0;0.12;0.15;0.35;0.625;0.75;0.87;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>

          {/* Status: SLEEPING */}
          <text x="646" y="120" fill={c.label} fontSize="7" opacity="1">
            SLEEPING
            <animate
              attributeName="opacity"
              values="1;1;0;0;0;0;0;1"
              keyTimes="0;0.12;0.13;0.14;0.86;0.87;0.87;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
          <text x="646" y="120" fill={o} fontSize="7" opacity="0">
            STARTING...
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0;0;0"
              keyTimes="0;0.13;0.14;0.34;0.35;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
          <text x="646" y="120" fill={g} fontSize="7" opacity="0">
            RUNNING
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0;0;0"
              keyTimes="0;0.35;0.36;0.74;0.75;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
          <text x="646" y="120" fill={r} fontSize="7" opacity="0">
            STOPPED
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0"
              keyTimes="0;0.75;0.76;0.87;0.88;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>

          {/* Idle timer */}
          <g opacity="0">
            <animate
              attributeName="opacity"
              values="0;0;1;1;0;0"
              keyTimes="0;0.63;0.64;0.75;0.76;1"
              dur="8s"
              repeatCount="indefinite"
            />
            <rect
              x="635"
              y="140"
              width="110"
              height="20"
              rx="3"
              fill={c.sec}
              stroke={c.border}
              strokeWidth="0.5"
            />
            <text
              x="690"
              y="154"
              textAnchor="middle"
              fill={c.label}
              fontSize="7"
            >
              wait_period
            </text>
            <rect
              x="637"
              y="155"
              width="106"
              height="3"
              rx="1"
              fill={c.border}
            />
            <rect x="637" y="155" width="106" height="3" rx="1" fill={r}>
              <animate
                attributeName="width"
                values="106;0"
                dur="8s"
                repeatCount="indefinite"
                keyTimes="0;1"
                calcMode="linear"
              />
              <animate
                attributeName="opacity"
                values="0;0;1;1;0;0"
                keyTimes="0;0.63;0.64;0.75;0.76;1"
                dur="8s"
                repeatCount="indefinite"
              />
            </rect>
          </g>

          <text
            x="690"
            y="200"
            textAnchor="middle"
            fill={c.label}
            fontSize="10"
          >
            SERVER
          </text>
        </g>

        {/* === BOTTOM TIMELINE === */}
        <line
          x1="40"
          y1="260"
          x2="760"
          y2="260"
          stroke={c.border}
          strokeWidth="0.5"
        />

        <g>
          <circle cx="130" cy="260" r="3" fill={o} opacity="0.3">
            <animate
              attributeName="opacity"
              values="0.3;0.3;1;1;0.3;0.3;0.3;0.3"
              keyTimes="0;0.01;0.02;0.12;0.13;1;1;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </circle>
          <text
            x="130"
            y="278"
            textAnchor="middle"
            fill={c.label}
            fontSize="7"
            letterSpacing="0.04em"
          >
            REQUEST ARRIVES
            <animate
              attributeName="fill"
              values={`${c.label};${c.label};${o};${o};${c.label};${c.label}`}
              keyTimes="0;0.01;0.02;0.12;0.13;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
        </g>

        <g>
          <circle cx="320" cy="260" r="3" fill={o} opacity="0.3">
            <animate
              attributeName="opacity"
              values="0.3;0.3;1;1;0.3;0.3"
              keyTimes="0;0.12;0.13;0.35;0.36;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </circle>
          <text
            x="320"
            y="278"
            textAnchor="middle"
            fill={c.label}
            fontSize="7"
            letterSpacing="0.04em"
          >
            PENNY STARTS APP
            <animate
              attributeName="fill"
              values={`${c.label};${c.label};${o};${o};${c.label};${c.label}`}
              keyTimes="0;0.12;0.13;0.35;0.36;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
        </g>

        <g>
          <circle cx="510" cy="260" r="3" fill={g} opacity="0.3">
            <animate
              attributeName="opacity"
              values="0.3;0.3;1;1;0.3;0.3"
              keyTimes="0;0.35;0.36;0.625;0.63;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </circle>
          <text
            x="510"
            y="278"
            textAnchor="middle"
            fill={c.label}
            fontSize="7"
            letterSpacing="0.04em"
          >
            TRAFFIC PROXIED
            <animate
              attributeName="fill"
              values={`${c.label};${c.label};${g};${g};${c.label};${c.label}`}
              keyTimes="0;0.35;0.36;0.625;0.63;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
        </g>

        <g>
          <circle cx="690" cy="260" r="3" fill={r} opacity="0.3">
            <animate
              attributeName="opacity"
              values="0.3;0.3;1;1;0.3;0.3"
              keyTimes="0;0.75;0.76;0.87;0.88;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </circle>
          <text
            x="690"
            y="278"
            textAnchor="middle"
            fill={c.label}
            fontSize="7"
            letterSpacing="0.04em"
          >
            {`IDLE \u2192 KILLED`}
            <animate
              attributeName="fill"
              values={`${c.label};${c.label};${r};${r};${c.label};${c.label}`}
              keyTimes="0;0.75;0.76;0.87;0.88;1"
              dur="8s"
              repeatCount="indefinite"
            />
          </text>
        </g>

        {/* Progress indicator */}
        <circle cx="40" cy="260" r="2" fill={o}>
          <animate
            attributeName="cx"
            values="40;760"
            dur="8s"
            repeatCount="indefinite"
            calcMode="linear"
          />
        </circle>
      </svg>
    </div>
  );
}
