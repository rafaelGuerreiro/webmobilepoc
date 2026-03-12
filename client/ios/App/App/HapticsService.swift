import Foundation
import UIKit

final class HapticsService {
    private let lightGenerator = UIImpactFeedbackGenerator(style: .light)
    private let hardGenerator = UIImpactFeedbackGenerator(style: .heavy)

    func prepare() {
        lightGenerator.prepare()
        hardGenerator.prepare()
    }

    func light() {
        lightGenerator.impactOccurred()
        lightGenerator.prepare()
    }

    func hard() {
        hardGenerator.impactOccurred()
        hardGenerator.prepare()
    }
}
