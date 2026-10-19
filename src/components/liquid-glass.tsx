/**
 * Liquid Glass 굴절 필터.
 *
 * 가운데는 그대로 두고 가장자리로 갈수록 뒤 배경을 바깥쪽으로 휘게 해서, 렌즈처럼 굴절되어 보이게 한다.
 * - 변위 지도: 빨강 = 가로 이동, 초록 = 세로 이동. 회색(128)은 움직이지 않음.
 *   가로·세로 그라데이션 위에 흐린 회색 사각형을 덮어 가운데만 중립으로 만든다.
 * - `backdrop-filter: url(#liquid-glass)`는 Chromium(Edge·WebView2)에서만 동작한다.
 *   다른 브라우저는 이 줄을 무시하고 일반 흐림(.glass)만 보인다.
 * 페이지에 한 번만 넣는다.
 */
const map = encodeURIComponent(
  `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" preserveAspectRatio="none">` +
    `<defs>` +
    `<linearGradient id="r" x1="0" x2="1" y1="0" y2="0"><stop offset="0" stop-color="#000"/><stop offset="1" stop-color="#f00"/></linearGradient>` +
    `<linearGradient id="g" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="#000"/><stop offset="1" stop-color="#0f0"/></linearGradient>` +
    `<filter id="b" x="-20%" y="-20%" width="140%" height="140%"><feGaussianBlur stdDeviation="7"/></filter>` +
    `</defs>` +
    `<rect width="100" height="100" fill="url(#r)"/>` +
    `<rect width="100" height="100" fill="url(#g)" style="mix-blend-mode:screen"/>` +
    `<rect x="14" y="14" width="72" height="72" rx="18" fill="#808080" filter="url(#b)"/>` +
    `</svg>`,
)

export function LiquidGlassFilter() {
  return (
    <svg aria-hidden width="0" height="0" className="pointer-events-none absolute">
      <filter
        id="liquid-glass"
        x="0"
        y="0"
        width="100%"
        height="100%"
        filterUnits="objectBoundingBox"
        colorInterpolationFilters="sRGB"
      >
        <feImage
          href={`data:image/svg+xml;utf8,${map}`}
          x="0"
          y="0"
          width="100%"
          height="100%"
          preserveAspectRatio="none"
          result="map"
        />
        <feDisplacementMap in="SourceGraphic" in2="map" scale="56" xChannelSelector="R" yChannelSelector="G" />
      </filter>
    </svg>
  )
}
