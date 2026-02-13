export function PennyLogo({
  size = 24,
  color = "currentColor",
  className,
}: {
  size?: number;
  color?: string;
  className?: string;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
      aria-label="Penny logo"
    >
      <title>Penny logo</title>
      <circle
        cx="50"
        cy="50"
        r="45"
        stroke={color}
        strokeWidth="6"
        fill="none"
      />
      <circle
        cx="50"
        cy="50"
        r="35"
        stroke={color}
        strokeWidth="3"
        fill="none"
      />
      <text
        x="50"
        y="50"
        textAnchor="middle"
        dominantBaseline="central"
        fill={color}
        fontSize="36"
        fontWeight="bold"
        fontFamily="system-ui, sans-serif"
      >
        P
      </text>
    </svg>
  );
}
