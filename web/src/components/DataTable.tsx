import React, { useState } from 'react';
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  getPaginationRowModel,
  getFilteredRowModel,
  useReactTable,
  SortingState,
} from '@tanstack/react-table';
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Search,
} from 'lucide-react';

interface DataTableProps<TData> {
  data: TData[];
  columns: ColumnDef<TData, any>[];
  pageSize?: number;
  searchPlaceholder?: string;
  showSearch?: boolean;
  emptyText?: string;
  lang?: 'vi' | 'en';
}

export function DataTable<TData>({
  data,
  columns,
  pageSize = 10,
  searchPlaceholder = 'Tìm kiếm dữ liệu...',
  showSearch = true,
  emptyText = 'Không tìm thấy dữ liệu',
  lang = 'vi',
}: DataTableProps<TData>) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [globalFilter, setGlobalFilter] = useState('');

  const table = useReactTable({
    data,
    columns,
    state: {
      sorting,
      globalFilter,
    },
    initialState: {
      pagination: {
        pageSize,
      },
    },
    onSortingChange: setSorting,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });

  const isVi = lang === 'vi';

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 14, width: '100%' }}>
      {/* Search and Table Toolbar */}
      {showSearch && (
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: 10 }}>
          <div style={{ position: 'relative', minWidth: 260 }}>
            <Search
              size={15}
              color="#94a3b8"
              style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)' }}
            />
            <input
              type="text"
              value={globalFilter ?? ''}
              onChange={(e) => setGlobalFilter(e.target.value)}
              placeholder={searchPlaceholder}
              style={{
                width: '100%',
                background: '#070b14',
                border: '1px solid #1e293b',
                color: '#fff',
                padding: '8px 12px 8px 36px',
                borderRadius: 8,
                fontSize: '0.84rem',
                outline: 'none',
              }}
            />
          </div>

          <div style={{ fontSize: '0.82rem', color: '#94a3b8' }}>
            {isVi ? `Hiển thị ${table.getRowModel().rows.length} / ${data.length} dòng` : `Showing ${table.getRowModel().rows.length} of ${data.length} rows`}
          </div>
        </div>
      )}

      {/* Main Table Container */}
      <div style={{
        overflowX: 'auto',
        borderRadius: 12,
        border: '1px solid #1e293b',
        background: '#0b1120',
      }}>
        <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.86rem' }}>
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id} style={{ borderBottom: '1px solid #1e293b', background: 'rgba(15, 23, 42, 0.8)' }}>
                {headerGroup.headers.map((header) => {
                  const canSort = header.column.getCanSort();
                  const isSorted = header.column.getIsSorted();

                  return (
                    <th
                      key={header.id}
                      onClick={header.column.getToggleSortingHandler()}
                      style={{
                        textAlign: 'left',
                        padding: '12px 16px',
                        color: '#94a3b8',
                        fontWeight: 600,
                        cursor: canSort ? 'pointer' : 'default',
                        userSelect: 'none',
                        whiteSpace: 'nowrap',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                        {flexRender(header.column.columnDef.header, header.getContext())}
                        {canSort && (
                          <span style={{ color: isSorted ? '#38bdf8' : '#475569' }}>
                            {isSorted === 'asc' ? (
                              <ArrowUp size={14} />
                            ) : isSorted === 'desc' ? (
                              <ArrowDown size={14} />
                            ) : (
                              <ArrowUpDown size={13} />
                            )}
                          </span>
                        )}
                      </div>
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.length === 0 ? (
              <tr>
                <td
                  colSpan={columns.length}
                  style={{
                    textAlign: 'center',
                    padding: '36px 16px',
                    color: '#64748b',
                    fontSize: '0.9rem',
                  }}
                >
                  {emptyText}
                </td>
              </tr>
            ) : (
              table.getRowModel().rows.map((row) => (
                <tr
                  key={row.id}
                  style={{
                    borderBottom: '1px solid #1e293b',
                    transition: 'background 0.15s ease',
                  }}
                  onMouseEnter={(e) => (e.currentTarget.style.background = 'rgba(30, 41, 59, 0.4)')}
                  onMouseLeave={(e) => (e.currentTarget.style.background = 'transparent')}
                >
                  {row.getVisibleCells().map((cell) => (
                    <td key={cell.id} style={{ padding: '12px 16px', color: '#f1f5f9' }}>
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination Controls */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: 12,
        padding: '6px 4px',
        fontSize: '0.82rem',
        color: '#94a3b8',
      }}>
        {/* Page size dropdown */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <span>{isVi ? 'Số dòng mỗi trang:' : 'Rows per page:'}</span>
          <select
            value={table.getState().pagination.pageSize}
            onChange={(e) => table.setPageSize(Number(e.target.value))}
            style={{
              background: '#070b14',
              border: '1px solid #1e293b',
              color: '#f8fafc',
              padding: '4px 8px',
              borderRadius: 6,
              fontSize: '0.82rem',
              outline: 'none',
              cursor: 'pointer',
            }}
          >
            {[10, 25, 50, 100].map((sz) => (
              <option key={sz} value={sz}>
                {sz}
              </option>
            ))}
          </select>
        </div>

        {/* Page indicator & navigation buttons */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ marginRight: 6 }}>
            {isVi
              ? `Trang ${table.getState().pagination.pageIndex + 1} / ${Math.max(1, table.getPageCount())}`
              : `Page ${table.getState().pagination.pageIndex + 1} of ${Math.max(1, table.getPageCount())}`}
          </span>

          <button
            onClick={() => table.setPageIndex(0)}
            disabled={!table.getCanPreviousPage()}
            style={{
              background: '#1e293b',
              border: '1px solid #334155',
              color: '#f8fafc',
              padding: '5px',
              borderRadius: 6,
              display: 'flex',
              alignItems: 'center',
              cursor: table.getCanPreviousPage() ? 'pointer' : 'not-allowed',
              opacity: table.getCanPreviousPage() ? 1 : 0.4,
            }}
            title={isVi ? 'Trang đầu' : 'First page'}
          >
            <ChevronsLeft size={16} />
          </button>

          <button
            onClick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
            style={{
              background: '#1e293b',
              border: '1px solid #334155',
              color: '#f8fafc',
              padding: '5px',
              borderRadius: 6,
              display: 'flex',
              alignItems: 'center',
              cursor: table.getCanPreviousPage() ? 'pointer' : 'not-allowed',
              opacity: table.getCanPreviousPage() ? 1 : 0.4,
            }}
            title={isVi ? 'Trang trước' : 'Previous page'}
          >
            <ChevronLeft size={16} />
          </button>

          <button
            onClick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
            style={{
              background: '#1e293b',
              border: '1px solid #334155',
              color: '#f8fafc',
              padding: '5px',
              borderRadius: 6,
              display: 'flex',
              alignItems: 'center',
              cursor: table.getCanNextPage() ? 'pointer' : 'not-allowed',
              opacity: table.getCanNextPage() ? 1 : 0.4,
            }}
            title={isVi ? 'Trang sau' : 'Next page'}
          >
            <ChevronRight size={16} />
          </button>

          <button
            onClick={() => table.setPageIndex(table.getPageCount() - 1)}
            disabled={!table.getCanNextPage()}
            style={{
              background: '#1e293b',
              border: '1px solid #334155',
              color: '#f8fafc',
              padding: '5px',
              borderRadius: 6,
              display: 'flex',
              alignItems: 'center',
              cursor: table.getCanNextPage() ? 'pointer' : 'not-allowed',
              opacity: table.getCanNextPage() ? 1 : 0.4,
            }}
            title={isVi ? 'Trang cuối' : 'Last page'}
          >
            <ChevronsRight size={16} />
          </button>
        </div>
      </div>
    </div>
  );
}
