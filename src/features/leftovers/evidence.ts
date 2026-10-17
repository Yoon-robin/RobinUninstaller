import type { Evidence, LeftoverKind } from "@/lib/api"

/** 근거를 사람이 읽을 문장으로. `warn`이면 경고색으로 보여 준다. */
export function describeEvidence(e: Evidence): { label: string; warn?: boolean } {
  switch (e.kind) {
    case "installLocation":
      return { label: "등록된 설치 폴더" }
    case "pointsIntoInstallLocation":
      return { label: "설치 폴더를 가리킴" }
    case "uninstallEntry":
      return { label: "남은 제거 등록 정보" }
    case "msiProductCode":
      return { label: "MSI 제품 코드 일치" }
    case "publisherAndProduct":
      return { label: "게시자 › 제품 이름 일치" }
    case "publisherAndPartialName":
      return { label: "게시자 아래 비슷한 이름 (다른 버전일 수 있음)" }
    case "exactName":
      return { label: "이름 일치" }
    case "partialName":
      return { label: "이름 일부 일치" }
    case "emptyPublisherFolder":
      return { label: "다른 제품 없는 게시자 폴더" }
    case "trackedInstall":
      return { label: "설치 추적 기록에 있음" }
    case "inUserFolder":
      return { label: "바탕 화면·문서 같은 사용자 폴더 안이에요", warn: true }
    case "sharedWithOtherApp":
      return { label: `다른 프로그램(${e.appName})과 함께 쓰는 것 같아요`, warn: true }
    case "containsUserDocuments":
      return { label: `직접 만든 문서 ${e.count}개 포함`, warn: true }
  }
}

export const kindLabels: Record<LeftoverKind, string> = {
  folder: "폴더",
  file: "파일",
  registryKey: "레지스트리 키",
  registryValue: "레지스트리 값",
}
