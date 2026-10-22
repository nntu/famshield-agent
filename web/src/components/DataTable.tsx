import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  type SortingState,
  useReactTable,
} from '@tanstack/react-table';
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Search,
} from 'lucide-react';
import { useId, useState } from 'react';

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
  const pageSizeId = useId();

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
    autoResetPageIndex: false,
    onSortingChange: setSorting,
    onGlobalFilterChange: setGlobalFilter,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });

  const isVi = lang === 'vi';

  return (
    <div className="flex flex-col gap-3.5 w-full">
      {/* Search and Table Toolbar */}
      {showSearch && (
        <div className="flex justify-between items-center flex-wrap gap-2.5">
          <div className="relative w-full sm:w-auto sm:min-w-[260px]">
            <Search
              size={15}
              aria-hidden="true"
              className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400"
            />
            <input
              type="text"
              aria-label={searchPlaceholder}
              value={globalFilter ?? ''}
              onChange={(e) => setGlobalFilter(e.target.value)}
              placeholder={searchPlaceholder}
              className="w-full bg-white border border-slate-200 text-slate-800 placeholder-slate-400 pl-9 pr-3 py-2 rounded-lg text-sm outline-none focus:border-sky-500 focus:ring-1 focus:ring-sky-500 transition-all shadow-xs"
            />
          </div>

          <div className="text-xs font-medium text-slate-500">
            {isVi
              ? `Hiển thị ${table.getRowModel().rows.length} / ${data.length} dòng`
              : `Showing ${table.getRowModel().rows.length} of ${data.length} rows`}
          </div>
        </div>
      )}

      {/* Main Table Container */}
      <div className="overflow-x-auto rounded-xl border border-slate-200 bg-white shadow-xs">
        <table className="w-full border-collapse text-sm text-left">
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id} className="border-b border-slate-200 bg-slate-50/80">
                {headerGroup.headers.map((header) => {
                  const canSort = header.column.getCanSort();
                  const isSorted = header.column.getIsSorted();

                  return (
                    <th
                      key={header.id}
                      aria-sort={
                        isSorted === 'asc'
                          ? 'ascending'
                          : isSorted === 'desc'
                            ? 'descending'
                            : canSort
                              ? 'none'
                              : undefined
                      }
                      className={`text-left px-4 py-3 text-slate-600 font-semibold whitespace-nowrap select-none ${
                        canSort ? 'hover:bg-slate-100/70' : ''
                      }`}
                    >
                      {canSort ? (
                        <button
                          type="button"
                          onClick={header.column.getToggleSortingHandler()}
                          className="flex w-full items-center gap-1.5 rounded-sm text-left outline-none focus-visible:ring-2 focus-visible:ring-sky-500 focus-visible:ring-offset-2"
                        >
                          {flexRender(header.column.columnDef.header, header.getContext())}
                          <span
                            aria-hidden="true"
                            className={isSorted ? 'text-sky-600' : 'text-slate-400'}
                          >
                            {isSorted === 'asc' ? (
                              <ArrowUp size={14} />
                            ) : isSorted === 'desc' ? (
                              <ArrowDown size={14} />
                            ) : (
                              <ArrowUpDown size={13} />
                            )}
                          </span>
                        </button>
                      ) : (
                        <div className="flex items-center gap-1.5">
                          {flexRender(header.column.columnDef.header, header.getContext())}
                        </div>
                      )}
                    </th>
                  );
                })}
              </tr>
            ))}
          </thead>
          <tbody className="divide-y divide-slate-100">
            {table.getRowModel().rows.length === 0 ? (
              <tr>
                <td
                  colSpan={columns.length}
                  className="text-center py-10 px-4 text-slate-500 text-sm"
                >
                  {emptyText}
                </td>
              </tr>
            ) : (
              table.getRowModel().rows.map((row) => (
                <tr key={row.id} className="hover:bg-slate-50/80 transition-colors">
                  {row.getVisibleCells().map((cell) => (
                    <td key={cell.id} className="px-4 py-3 text-slate-700">
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
      <div className="flex justify-between items-center flex-wrap gap-3 px-1 py-1.5 text-xs text-slate-500">
        {/* Page size dropdown */}
        <div className="flex items-center gap-2">
          <label htmlFor={pageSizeId}>{isVi ? 'Số dòng mỗi trang:' : 'Rows per page:'}</label>
          <select
            id={pageSizeId}
            value={table.getState().pagination.pageSize}
            onChange={(e) => table.setPageSize(Number(e.target.value))}
            className="bg-white border border-slate-200 text-slate-700 px-2 py-1 rounded-md text-xs outline-none cursor-pointer focus:border-sky-500 shadow-xs"
          >
            {[10, 25, 50, 100].map((sz) => (
              <option key={sz} value={sz}>
                {sz}
              </option>
            ))}
          </select>
        </div>

        {/* Page indicator & navigation buttons */}
        <div className="flex items-center gap-1.5">
          <span className="mr-1.5 font-medium">
            {isVi
              ? `Trang ${table.getState().pagination.pageIndex + 1} / ${Math.max(1, table.getPageCount())}`
              : `Page ${table.getState().pagination.pageIndex + 1} of ${Math.max(1, table.getPageCount())}`}
          </span>

          <button
            onClick={() => table.setPageIndex(0)}
            disabled={!table.getCanPreviousPage()}
            className="bg-white hover:bg-slate-50 disabled:hover:bg-white border border-slate-200 text-slate-700 p-1.5 rounded-md flex items-center cursor-pointer disabled:cursor-not-allowed disabled:opacity-40 transition-colors shadow-xs"
            title={isVi ? 'Trang đầu' : 'First page'}
            aria-label={isVi ? 'Trang đầu' : 'First page'}
          >
            <ChevronsLeft size={15} />
          </button>
          <button
            onClick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
            className="bg-white hover:bg-slate-50 disabled:hover:bg-white border border-slate-200 text-slate-700 p-1.5 rounded-md flex items-center cursor-pointer disabled:cursor-not-allowed disabled:opacity-40 transition-colors shadow-xs"
            title={isVi ? 'Trang trước' : 'Previous page'}
            aria-label={isVi ? 'Trang trước' : 'Previous page'}
          >
            <ChevronLeft size={15} />
          </button>
          <button
            onClick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
            className="bg-white hover:bg-slate-50 disabled:hover:bg-white border border-slate-200 text-slate-700 p-1.5 rounded-md flex items-center cursor-pointer disabled:cursor-not-allowed disabled:opacity-40 transition-colors shadow-xs"
            title={isVi ? 'Trang sau' : 'Next page'}
            aria-label={isVi ? 'Trang sau' : 'Next page'}
          >
            <ChevronRight size={15} />
          </button>
          <button
            onClick={() => table.setPageIndex(table.getPageCount() - 1)}
            disabled={!table.getCanNextPage()}
            className="bg-white hover:bg-slate-50 disabled:hover:bg-white border border-slate-200 text-slate-700 p-1.5 rounded-md flex items-center cursor-pointer disabled:cursor-not-allowed disabled:opacity-40 transition-colors shadow-xs"
            title={isVi ? 'Trang cuối' : 'Last page'}
            aria-label={isVi ? 'Trang cuối' : 'Last page'}
          >
            <ChevronsRight size={15} />
          </button>
        </div>
      </div>
    </div>
  );
}
