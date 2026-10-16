import { useMemo, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { ArrowDownAZ, HardDrive, MoreHorizontal, Package, RefreshCw, Search, Trash2, User } from "lucide-react"

import { api, type InstalledApplication } from "@/lib/api"
import { formatInstallDate, formatSize, installDateSortKey } from "@/lib/format"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import { Skeleton } from "@/components/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"

type SortKey = "name" | "size" | "date"

const sortLabels: Record<SortKey, string> = {
  name: "이름순",
  size: "크기순",
  date: "설치일순",
}

function compare(a: InstalledApplication, b: InstalledApplication, key: SortKey) {
  switch (key) {
    case "size":
      return (b.estimatedSizeKb ?? 0) - (a.estimatedSizeKb ?? 0)
    case "date":
      return installDateSortKey(b.installDate).localeCompare(installDateSortKey(a.installDate))
    default:
      return a.displayName.localeCompare(b.displayName, "ko")
  }
}

export function ProgramsPage() {
  const [query, setQuery] = useState("")
  const [sort, setSort] = useState<SortKey>("name")
  const apps = useQuery({ queryKey: ["installed-apps"], queryFn: api.listInstalledApps })

  const visible = useMemo(() => {
    const q = query.trim().toLowerCase()
    return (apps.data ?? [])
      .filter(
        (a) =>
          !q ||
          a.displayName.toLowerCase().includes(q) ||
          (a.publisher ?? "").toLowerCase().includes(q),
      )
      .sort((a, b) => compare(a, b, sort))
  }, [apps.data, query, sort])

  const stats = useMemo(() => {
    const list = apps.data ?? []
    return {
      total: list.length,
      totalKb: list.reduce((sum, a) => sum + (a.estimatedSizeKb ?? 0), 0),
      user: list.filter((a) => a.scope === "user").length,
    }
  }, [apps.data])

  return (
    <div className="flex min-w-0 flex-col gap-6 p-6">
      <div className="grid gap-4 sm:grid-cols-3">
        <StatCard icon={Package} label="설치된 프로그램" value={apps.isLoading ? null : `${stats.total}개`} />
        <StatCard icon={HardDrive} label="전체 예상 용량" value={apps.isLoading ? null : formatSize(stats.totalKb)} />
        <StatCard icon={User} label="현재 사용자 전용" value={apps.isLoading ? null : `${stats.user}개`} />
      </div>

      <Card>
        <CardHeader className="flex flex-row flex-wrap items-center justify-between gap-3">
          <div>
            <CardTitle>설치된 프로그램</CardTitle>
            <CardDescription>
              {apps.isLoading ? "목록을 불러오는 중…" : `${visible.length}개 표시 중`}
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <div className="relative">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="이름 또는 게시자 검색"
                className="w-56 pl-8"
              />
            </div>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline">
                  <ArrowDownAZ />
                  {sortLabels[sort]}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {(Object.keys(sortLabels) as SortKey[]).map((key) => (
                  <DropdownMenuItem key={key} onClick={() => setSort(key)}>
                    {sortLabels[key]}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
            <Button variant="outline" size="icon" onClick={() => apps.refetch()} aria-label="새로고침">
              <RefreshCw className={apps.isFetching ? "animate-spin" : undefined} />
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          {apps.isError ? (
            <p className="py-10 text-center text-sm text-destructive">
              목록을 불러오지 못했어요: {String(apps.error)}
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>이름</TableHead>
                  <TableHead>게시자</TableHead>
                  <TableHead>버전</TableHead>
                  <TableHead className="text-right">크기</TableHead>
                  <TableHead>설치일</TableHead>
                  <TableHead>형식</TableHead>
                  <TableHead className="w-10" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {apps.isLoading
                  ? Array.from({ length: 8 }, (_, i) => (
                      <TableRow key={i}>
                        <TableCell colSpan={7}>
                          <Skeleton className="h-5 w-full" />
                        </TableCell>
                      </TableRow>
                    ))
                  : visible.map((app) => <ProgramRow key={app.id} app={app} />)}
                {!apps.isLoading && visible.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={7} className="py-10 text-center text-muted-foreground">
                      검색 결과가 없어요
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function ProgramRow({ app }: { app: InstalledApplication }) {
  return (
    <TableRow>
      <TableCell className="max-w-60 truncate font-medium" title={app.displayName}>
        {app.displayName}
      </TableCell>
      <TableCell className="max-w-36 truncate text-muted-foreground" title={app.publisher ?? undefined}>
        {app.publisher ?? "—"}
      </TableCell>
      <TableCell className="max-w-28 truncate text-muted-foreground">{app.version ?? "—"}</TableCell>
      <TableCell className="text-right tabular-nums">{formatSize(app.estimatedSizeKb)}</TableCell>
      <TableCell className="tabular-nums text-muted-foreground">{formatInstallDate(app.installDate)}</TableCell>
      <TableCell>
        <div className="flex gap-1">
          <Badge variant={app.technology === "msi" ? "secondary" : "outline"}>
            {app.technology.toUpperCase()}
          </Badge>
          {app.scope === "machine32" && <Badge variant="outline">32bit</Badge>}
          {app.scope === "user" && <Badge variant="outline">사용자</Badge>}
        </div>
      </TableCell>
      <TableCell>
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" aria-label="작업">
              <MoreHorizontal />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuLabel className="max-w-64 truncate">{app.displayName}</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem disabled>
              <Trash2 />
              제거 (Phase 2에서 지원)
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </TableCell>
    </TableRow>
  )
}

function StatCard({ icon: Icon, label, value }: { icon: typeof Package; label: string; value: string | null }) {
  return (
    <Card>
      <CardContent className="flex items-center gap-4">
        <div className="flex size-10 items-center justify-center rounded-lg bg-muted">
          <Icon className="size-5 text-muted-foreground" />
        </div>
        <div>
          <p className="text-sm text-muted-foreground">{label}</p>
          {value == null ? <Skeleton className="mt-1 h-6 w-16" /> : <p className="text-xl font-semibold">{value}</p>}
        </div>
      </CardContent>
    </Card>
  )
}
