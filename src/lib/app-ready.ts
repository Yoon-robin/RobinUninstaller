import { createContext, useContext } from "react"

/**
 * 시작 화면이 걷혔는지. 처음 켤 때 목록 등장·막대·숫자 올라가기가 시작 화면 뒤에서 끝나 버리지 않게,
 * 이 값이 true가 된 뒤에 움직임을 시작한다. 시작 화면을 쓰지 않는 곳(설치 화면, 화면 캡처)에서는 true.
 */
export const AppReadyContext = createContext(true)

export const useAppReady = () => useContext(AppReadyContext)
