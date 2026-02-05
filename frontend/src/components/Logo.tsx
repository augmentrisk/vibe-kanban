export function Logo() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 920 180" width="140" role="img" aria-label="MultiVibe logo" className="logo">
      <defs>
        <g id="echo" fill="none" stroke="#0b0f1a" strokeWidth="1.5" shapeRendering="crispEdges">
          <use href="#word" transform="translate(2,2)"/>
          <use href="#word" transform="translate(4,4)"/>
          <use href="#word" transform="translate(6,6)"/>
        </g>
      </defs>

      <g transform="translate(60,28)">
        <g id="word" shapeRendering="crispEdges">

          {/* M (two legs, V-connected) */}
          <g transform="translate(0,0)">
            {/* outer legs */}
            <rect x="0"  y="0" width="16" height="92"/>
            <rect x="52" y="0" width="16" height="92"/>

            {/* inner diagonals */}
            <rect x="16" y="12" width="12" height="56"/>
            <rect x="40" y="12" width="12" height="56"/>

            {/* V connection (THIS is the key shape) */}
            <rect x="28" y="66" width="12" height="14"/>

            {/* bottom feet */}
            <rect x="0"  y="78" width="24" height="14"/>
            <rect x="44" y="78" width="24" height="14"/>
          </g>

          {/* U */}
          <g transform="translate(92,0)">
            <rect x="0"  y="0" width="16" height="80"/>
            <rect x="52" y="0" width="16" height="80"/>
            <rect x="10" y="80" width="58" height="12"/>
          </g>

          {/* L */}
          <g transform="translate(184,0)">
            <rect x="0" y="0" width="16" height="92"/>
            <rect x="10" y="80" width="66" height="12"/>
          </g>

          {/* T */}
          <g transform="translate(276,0)">
            <rect x="0" y="0" width="84" height="16"/>
            <rect x="34" y="12" width="16" height="80"/>
          </g>

          {/* I */}
          <g transform="translate(380,0)">
            <rect x="26" y="0" width="16" height="92"/>
          </g>

          {/* V */}
          <g transform="translate(448,0)">
            <rect x="0"  y="0" width="16" height="66"/>
            <rect x="52" y="0" width="16" height="66"/>
            <rect x="14" y="64" width="14" height="28"/>
            <rect x="40" y="64" width="14" height="28"/>
            <rect x="24" y="78" width="20" height="14"/>
          </g>

          {/* I */}
          <g transform="translate(540,0)">
            <rect x="26" y="0" width="16" height="92"/>
          </g>

          {/* B */}
          <g transform="translate(608,0)">
            <rect x="0" y="0" width="16" height="92"/>
            <rect x="12" y="0" width="62" height="16"/>
            <rect x="12" y="38" width="56" height="16"/>
            <rect x="12" y="76" width="62" height="16"/>
            <rect x="66" y="14" width="16" height="26"/>
            <rect x="60" y="52" width="22" height="26"/>
            <rect x="22" y="20" width="38" height="12" fill="white"/>
            <rect x="22" y="58" width="34" height="12" fill="white"/>
          </g>

          {/* E */}
          <g transform="translate(710,0)">
            <rect x="0" y="0" width="16" height="92"/>
            <rect x="12" y="0" width="76" height="16"/>
            <rect x="12" y="38" width="60" height="16"/>
            <rect x="12" y="76" width="76" height="16"/>
          </g>
        </g>

        {/* echo */}
        <use href="#echo"/>

        {/* front face */}
        <use href="#word" fill="#0b0f1a" stroke="none" shapeRendering="crispEdges"/>
      </g>

      {/* subtitle */}
      <g transform="translate(0,154)">
        <g transform="scale(1.35,1)">
          <text
            x="60"
            y="0"
            fontFamily="ui-monospace, Menlo, Consolas, monospace"
            fontSize="18"
            letterSpacing="2"
            fill="#0b0f1a"
            opacity="0.85"
          >
            a Vibe-Kanban fork
          </text>
        </g>
      </g>
    </svg>
  );
}
