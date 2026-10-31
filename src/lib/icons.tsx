interface IconProps {
  className?: string;
}

export function AppMark({ className }: IconProps) {
  return (
    <svg
      aria-hidden="true"
      className={className}
      fill="none"
      viewBox="0 0 32 32"
    >
      <path d="M4.5 4.5h23v23h-23z" stroke="currentColor" strokeWidth="1.25" />
      <path d="M4.5 11.5h23M11.5 4.5v23" opacity="0.38" stroke="currentColor" strokeWidth="1" />
      <path d="M11.5 11.5h16v16" stroke="currentColor" strokeWidth="2.25" />
      <path d="M20.5 20.5h7v7" stroke="currentColor" strokeWidth="1.25" />
    </svg>
  );
}

interface FrameDiagramProps {
  sourceWidth: number;
  sourceHeight: number;
  targetWidth: number;
  targetHeight: number;
}

export function FrameDiagram({
  sourceWidth,
  sourceHeight,
  targetWidth,
  targetHeight,
}: FrameDiagramProps) {
  const ratio = sourceWidth / sourceHeight;
  const targetRatio = targetWidth / targetHeight;
  const outputWidth = ratio >= targetRatio ? targetWidth : Math.round((targetHeight * ratio) / 2) * 2;
  const outputHeight = ratio >= targetRatio ? Math.round((targetWidth / ratio) / 2) * 2 : targetHeight;
  const padX = Math.round((targetWidth - outputWidth) / 2);
  const padY = Math.round((targetHeight - outputHeight) / 2);
  const noPadding = sourceWidth === targetWidth && sourceHeight === targetHeight;

  const maxWidth = 308;
  const maxHeight = 173.25;
  const frameWidth = targetRatio >= maxWidth / maxHeight ? maxWidth : maxHeight * targetRatio;
  const frameHeight = targetRatio >= maxWidth / maxHeight ? maxWidth / targetRatio : maxHeight;
  const scale = Math.min(frameWidth / targetWidth, frameHeight / targetHeight);
  const innerWidth = outputWidth * scale;
  const innerHeight = outputHeight * scale;
  const innerX = (frameWidth - innerWidth) / 2;
  const innerY = (frameHeight - innerHeight) / 2;

  return (
    <svg
      aria-hidden="true"
      className="frame-svg"
      fill="none"
      viewBox="0 0 360 220"
    >
      <text className="frame-svg-label" x="26" y="16">
        SOURCE {sourceWidth} × {sourceHeight}
      </text>
      <text className="frame-svg-label" x="334" y="16" textAnchor="end">
        CANVAS {targetWidth} × {targetHeight}
      </text>
      <g transform="translate(26 32)">
        <rect
          height={frameHeight}
          stroke="currentColor"
          strokeDasharray="2 4"
          strokeOpacity="0.42"
          width={frameWidth}
        />
        <rect
          fill="currentColor"
          fillOpacity="0.08"
          height={innerHeight}
          stroke="currentColor"
          strokeOpacity="0.9"
          width={innerWidth}
          x={innerX}
          y={innerY}
        />
        <path
          d={`M${innerX} ${innerY + 12}V${innerY}H${innerX + 12}`}
          stroke="currentColor"
          strokeWidth="2"
        />
        <path
          d={`M${innerX + innerWidth - 12} ${innerY + innerHeight}H${innerX + innerWidth}V${innerY + innerHeight - 12}`}
          stroke="currentColor"
          strokeWidth="2"
        />
      </g>
      <text className="frame-svg-meta" x="26" y="216">
        {noPadding ? "NO PADDING" : `PAD X ${padX} / Y ${padY}`}
      </text>
      <text className="frame-svg-meta" x="334" y="216" textAnchor="end">
        CONTENT {outputWidth} × {outputHeight}
      </text>
    </svg>
  );
}