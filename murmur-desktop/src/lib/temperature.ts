/** Temperature visual mapping.
 *
 * Cold (0-10):   grey, no animation
 * Warm (10-50):  orange, subtle glow
 * Hot (50-100):  orange-to-pink gradient, gentle pulse
 * On fire (100+): animated shimmer, larger
 */

export type TempLevel = "cold" | "warm" | "hot" | "fire";

export function getTempLevel(temperature: number): TempLevel {
  if (temperature >= 100) return "fire";
  if (temperature >= 50) return "hot";
  if (temperature >= 10) return "warm";
  return "cold";
}

export function getTempColour(temperature: number): string {
  const level = getTempLevel(temperature);
  switch (level) {
    case "cold":
      return "#ADB5BD";
    case "warm":
      return "#E8590C";
    case "hot":
      return "#C2255C";
    case "fire":
      return "#E8590C";
  }
}

export function getTempBgClass(temperature: number): string {
  const level = getTempLevel(temperature);
  switch (level) {
    case "cold":
      return "bg-gray-200 text-gray-600";
    case "warm":
      return "bg-orange-100 text-orange-700";
    case "hot":
      return "bg-gradient-to-r from-orange-400 to-pink-500 text-white";
    case "fire":
      return "bg-gradient-to-r from-orange-500 to-pink-600 text-white animate-pulse";
  }
}
