import os
import subprocess

# Variation 1: Elegant Ballet Pointe Shoe with Ribbons & Stage Arch
svg_v1 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <!-- Outer Rounded Squircle -->
  <rect x="40" y="40" width="944" height="944" rx="210" fill="#18181e" stroke="#ffb31a" stroke-width="26"/>
  
  <!-- Subtle Proscenium Dome Arch in Background -->
  <path d="M 210 700 C 210 280, 814 280, 814 700 Z" fill="#252632" opacity="0.7"/>

  <!-- Main Golden Disc -->
  <circle cx="512" cy="485" r="255" fill="#ffb31a"/>

  <!-- Five Bottom Theatrical Footlight / Equalizer Pillars -->
  <g fill="#ffb31a">
    <rect x="290" y="700" width="44" height="70" rx="22"/>
    <rect x="390" y="680" width="44" height="120" rx="22"/>
    <rect x="490" y="650" width="44" height="180" rx="22"/>
    <rect x="590" y="680" width="44" height="120" rx="22"/>
    <rect x="690" y="700" width="44" height="70" rx="22"/>
  </g>

  <!-- Ballet Pointe Shoe Silhouette in Dark Charcoal (#18181e) -->
  <g fill="#18181e">
    <!-- Star at top representing Principal Dancer / Reparto -->
    <polygon points="512,305 522,328 547,329 527,344 534,368 512,353 490,368 497,344 477,329 502,328"/>
    
    <!-- Left Ribbon Cross -->
    <path d="M 460 365 C 490 395, 534 425, 564 455 L 544 472 C 518 445, 478 418, 442 382 Z"/>
    <!-- Right Ribbon Cross -->
    <path d="M 564 365 C 534 395, 490 425, 460 455 L 480 472 C 506 445, 546 418, 582 382 Z"/>

    <!-- Left Pointe Shoe en pointe -->
    <path d="M 458 450 C 475 450, 488 475, 492 510 C 496 550, 490 600, 480 645 L 452 645 C 442 600, 436 550, 440 510 C 444 475, 450 450, 458 450 Z"/>

    <!-- Right Pointe Shoe en pointe -->
    <path d="M 566 450 C 550 450, 536 475, 532 510 C 528 550, 534 600, 544 645 L 572 645 C 582 600, 588 550, 584 510 C 580 475, 574 450, 566 450 Z"/>

    <!-- Platform Flat Tips (Box) -->
    <rect x="452" y="642" width="28" height="8" rx="3"/>
    <rect x="544" y="642" width="28" height="8" rx="3"/>
  </g>
</svg>
"""

# Variation 2: Bold Modern Monogram "TG" + Ballet & Theater Elements
svg_v2 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <!-- Outer Rounded Squircle -->
  <rect x="40" y="40" width="944" height="944" rx="210" fill="#18181e" stroke="#ffb31a" stroke-width="26"/>
  
  <!-- Subtle Proscenium Dome Arch in Background -->
  <path d="M 210 700 C 210 280, 814 280, 814 700 Z" fill="#252632" opacity="0.7"/>

  <!-- Main Golden Disc -->
  <circle cx="512" cy="485" r="255" fill="#ffb31a"/>

  <!-- Five Bottom Theatrical Footlight / Equalizer Pillars -->
  <g fill="#ffb31a">
    <rect x="290" y="700" width="44" height="70" rx="22"/>
    <rect x="390" y="680" width="44" height="120" rx="22"/>
    <rect x="490" y="650" width="44" height="180" rx="22"/>
    <rect x="590" y="680" width="44" height="120" rx="22"/>
    <rect x="690" y="700" width="44" height="70" rx="22"/>
  </g>

  <!-- Monogram TG in Dark Charcoal (#18181e) -->
  <g fill="#18181e">
    <!-- Star above TG -->
    <polygon points="512,285 522,308 547,309 527,324 534,348 512,333 490,348 497,324 477,309 502,308"/>

    <!-- Letter T -->
    <path d="M 330 365 L 490 365 L 490 425 L 436 425 L 436 635 L 384 635 L 384 425 L 330 425 Z"/>

    <!-- Letter G -->
    <path d="M 590 365 C 655 365, 695 405, 695 470 L 642 470 C 642 438, 620 420, 590 420 C 555 420, 530 445, 530 500 C 530 555, 555 580, 590 580 C 620 580, 642 562, 642 530 L 590 530 L 590 480 L 695 480 L 695 565 C 675 615, 640 635, 590 635 C 520 635, 476 580, 476 500 C 476 420, 520 365, 590 365 Z"/>
  </g>
</svg>
"""

# Variation 3: Stylized Theater Clipboard / Management Crest with Ballet Ribbon
svg_v3 = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <!-- Outer Rounded Squircle -->
  <rect x="40" y="40" width="944" height="944" rx="210" fill="#18181e" stroke="#ffb31a" stroke-width="26"/>
  
  <!-- Subtle Proscenium Dome Arch in Background -->
  <path d="M 210 700 C 210 280, 814 280, 814 700 Z" fill="#252632" opacity="0.7"/>

  <!-- Main Golden Disc -->
  <circle cx="512" cy="485" r="255" fill="#ffb31a"/>

  <!-- Five Bottom Theatrical Footlight / Equalizer Pillars -->
  <g fill="#ffb31a">
    <rect x="290" y="700" width="44" height="70" rx="22"/>
    <rect x="390" y="680" width="44" height="120" rx="22"/>
    <rect x="490" y="650" width="44" height="180" rx="22"/>
    <rect x="590" y="680" width="44" height="120" rx="22"/>
    <rect x="690" y="700" width="44" height="70" rx="22"/>
  </g>

  <!-- Stage Management Clipboard & Ballet Cross Emblem -->
  <g fill="#18181e">
    <!-- Clipboard top clip -->
    <rect x="462" y="315" width="100" height="30" rx="12"/>
    <rect x="482" y="295" width="60" height="25" rx="8"/>

    <!-- Clipboard Board Body -->
    <rect x="372" y="340" width="280" height="300" rx="24"/>
  </g>

  <!-- Golden Inner Checklist & Ballet Ribbon Detail on Board -->
  <g fill="#ffb31a">
    <!-- Star of Excellence -->
    <polygon points="420,380 425,392 438,393 428,401 432,413 420,405 408,413 412,401 402,393 415,392"/>
    <rect x="450" y="392" width="150" height="16" rx="8"/>

    <polygon points="420,440 425,452 438,453 428,461 432,473 420,465 408,473 412,461 402,453 415,452"/>
    <rect x="450" y="452" width="150" height="16" rx="8"/>

    <!-- Ballet Pointe Shoe silhouette on bottom of clipboard -->
    <path d="M 420,500 L 440,545 L 430,555 L 410,510 Z"/>
    <rect x="450" y="512" width="110" height="16" rx="8"/>
  </g>
</svg>
"""

with open('icon_v1.svg', 'w') as f:
    f.write(svg_v1)

with open('icon_v2.svg', 'w') as f:
    f.write(svg_v2)

with open('icon_v3.svg', 'w') as f:
    f.write(svg_v3)

for v in ['v1', 'v2', 'v3']:
    subprocess.run(['rsvg-convert', '-w', '512', '-h', '512', f'icon_{v}.svg', '-o', f'icon_{v}.png'], check=True)
print("Icons rendered successfully.")
