import { useMemo, useState } from "react";

import { AppSidebar } from "@/components/app-sidebar";
import {
  VAULT_ITEMS_PER_PAGE,
  VAULT_MOCK_DATA,
} from "@/components/vault/vault.mock-data";
import { VaultFilterBar } from "@/components/vault/vault-filter-bar";
import { VaultPagination } from "@/components/vault/vault-pagination";
import { VaultResultsList } from "@/components/vault/vault-results-list";
import { VaultSearchBar } from "@/components/vault/vault-search-bar";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";

const Vault = () => {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("All");
  const [showFilters, setShowFilters] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);

  const categories = useMemo(
    () => ["All", ...new Set(VAULT_MOCK_DATA.map((item) => item.category))],
    [],
  );

  const filteredData = useMemo(() => {
    return VAULT_MOCK_DATA.filter((item) => {
      const normalizedQuery = searchQuery.trim().toLowerCase();
      const matchesSearch =
        normalizedQuery.length === 0 ||
        item.question.toLowerCase().includes(normalizedQuery) ||
        item.answer.toLowerCase().includes(normalizedQuery);
      const matchesCategory =
        selectedCategory === "All" || item.category === selectedCategory;

      return matchesSearch && matchesCategory;
    });
  }, [searchQuery, selectedCategory]);

  const totalPages = Math.max(
    1,
    Math.ceil(filteredData.length / VAULT_ITEMS_PER_PAGE),
  );

  const safeCurrentPage = Math.min(currentPage, totalPages);
  const startIndex = (safeCurrentPage - 1) * VAULT_ITEMS_PER_PAGE;
  const endIndex = startIndex + VAULT_ITEMS_PER_PAGE;
  const paginatedData = filteredData.slice(startIndex, endIndex);

  const handleSearchChange = (value: string) => {
    setSearchQuery(value);
    setCurrentPage(1);
  };

  const handleCategoryChange = (category: string) => {
    setSelectedCategory(category);
    setCurrentPage(1);
  };

  return (
    <SidebarProvider className="[--sidebar:var(--background)] [--sidebar-accent:var(--background)] [--sidebar-border:var(--border)]">
      <AppSidebar />
      <SidebarInset className="bg-background">
        <div className="flex h-full w-full flex-col bg-background">
          <SidebarTrigger className="m-2" />

          <main className="min-h-0 flex-1 overflow-y-auto">
            <div className="mx-auto w-full max-w-[1200px] px-6 pb-8 pt-2">
              <section className="mb-8 space-y-4">
                <VaultSearchBar
                  value={searchQuery}
                  onValueChange={handleSearchChange}
                />

                <VaultFilterBar
                  showFilters={showFilters}
                  categories={categories}
                  selectedCategory={selectedCategory}
                  onToggleFilters={() => setShowFilters((prev) => !prev)}
                  onCategoryChange={handleCategoryChange}
                />
              </section>

              <div className="mb-4 text-sm text-muted-foreground">
                {filteredData.length} {filteredData.length === 1 ? "result" : "results"}
                {filteredData.length > VAULT_ITEMS_PER_PAGE ? (
                  <span className="ml-2">
                    • Page {safeCurrentPage} of {totalPages}
                  </span>
                ) : null}
              </div>

              <VaultResultsList items={paginatedData} />

              <VaultPagination
                currentPage={safeCurrentPage}
                totalPages={totalPages}
                totalItems={filteredData.length}
                startIndex={startIndex}
                endIndex={endIndex}
                onPageChange={setCurrentPage}
              />
            </div>
          </main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Vault;
