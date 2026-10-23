Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase
Add-Type -AssemblyName System.Windows.Forms   # «Examinar», selector de archivos y DoEvents (sin esto fallaban)
Add-Type -AssemblyName Microsoft.VisualBasic   # cuadro de «Días personalizados»

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$installerExe = Join-Path $scriptDir "Instalar_TeatroGestion.exe"
if (-not (Test-Path $installerExe)) {
    $alt = Join-Path $scriptDir "TeatroGestion\Instalar_TeatroGestion.exe"
    if (Test-Path $alt) { $installerExe = $alt }
}

[xml]$xaml = @"
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation"
        xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
        Title="TeatroGestión — Instalador Oficial Windows 11" 
        Height="720" Width="840" 
        WindowStartupLocation="CenterScreen" 
        ResizeMode="CanMinimize"
        Background="#121214" Foreground="#E0E0E0" FontFamily="Segoe UI">
    <Window.Resources>
        <Style TargetType="TextBlock">
            <Setter Property="Foreground" Value="#CCCCCC"/>
            <Setter Property="FontSize" Value="13"/>
        </Style>
        <Style TargetType="TextBox">
            <Setter Property="Background" Value="#1C1C22"/>
            <Setter Property="Foreground" Value="#FFFFFF"/>
            <Setter Property="BorderBrush" Value="#383842"/>
            <Setter Property="BorderThickness" Value="1"/>
            <Setter Property="Padding" Value="8,5"/>
            <Setter Property="FontSize" Value="13"/>
        </Style>
        <Style TargetType="PasswordBox">
            <Setter Property="Background" Value="#1C1C22"/>
            <Setter Property="Foreground" Value="#FFFFFF"/>
            <Setter Property="BorderBrush" Value="#383842"/>
            <Setter Property="BorderThickness" Value="1"/>
            <Setter Property="Padding" Value="8,5"/>
            <Setter Property="FontSize" Value="13"/>
        </Style>
        <Style TargetType="ComboBox">
            <Setter Property="Background" Value="#1C1C22"/>
            <Setter Property="Foreground" Value="#121214"/>
            <Setter Property="Padding" Value="6,4"/>
            <Setter Property="FontSize" Value="13"/>
        </Style>
        <Style TargetType="Button">
            <Setter Property="Background" Value="#FFB31A"/>
            <Setter Property="Foreground" Value="#121214"/>
            <Setter Property="FontWeight" Value="SemiBold"/>
            <Setter Property="BorderThickness" Value="0"/>
            <Setter Property="Padding" Value="16,8"/>
            <Setter Property="Cursor" Value="Hand"/>
        </Style>
        <Style TargetType="TabItem">
            <Setter Property="Foreground" Value="#A0A0B0"/>
            <Setter Property="Background" Value="#18181E"/>
            <Setter Property="FontSize" Value="14"/>
            <Setter Property="FontWeight" Value="SemiBold"/>
            <Setter Property="Padding" Value="20,10"/>
        </Style>
    </Window.Resources>

    <Grid Margin="20">
        <Grid.RowDefinitions>
            <RowDefinition Height="Auto"/>
            <RowDefinition Height="*"/>
            <RowDefinition Height="Auto"/>
            <RowDefinition Height="130"/>
        </Grid.RowDefinitions>

        <!-- Header -->
        <Border Grid.Row="0" Background="#18181E" BorderBrush="#2D2D36" BorderThickness="1" CornerRadius="8" Padding="18,12" Margin="0,0,0,15">
            <Grid>
                <Grid.ColumnDefinitions>
                    <ColumnDefinition Width="*"/>
                    <ColumnDefinition Width="Auto"/>
                </Grid.ColumnDefinitions>
                <StackPanel Grid.Column="0">
                    <TextBlock Text="TEATROGESTIÓN" FontSize="20" FontWeight="Bold" Foreground="#FFB31A"/>
                    <TextBlock Text="Ballet Oficial de Córdoba — Teatro del Libertador San Martín" FontSize="12" Foreground="#8E8E9F" Margin="0,2,0,4"/>
                    <TextBlock Text="★ REPARTO DE ESTUDIO, SIN ORDEN DE PRIORIDAD Y SUJETO A CAMBIOS. ★" FontSize="11" Foreground="#FFB31A" FontStyle="Italic"/>
                </StackPanel>
                <Border Grid.Column="1" Background="#262630" CornerRadius="4" Padding="10,4" VerticalAlignment="Center">
                    <TextBlock Text="WINDOWS 11 NATIVO" FontSize="11" FontWeight="Bold" Foreground="#00E676"/>
                </Border>
            </Grid>
        </Border>

        <!-- Main Tabs -->
        <TabControl Grid.Row="1" Background="#18181E" BorderBrush="#2D2D36" Margin="0,0,0,15">
            
            <!-- Tab 1: Instalación de TeatroGestión -->
            <TabItem Header="Instalación y Licencia">
                <ScrollViewer VerticalScrollBarVisibility="Auto" Padding="15">
                    <StackPanel>
                        <!-- Destino -->
                        <TextBlock Text="Carpeta de Instalación:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                        <Grid Margin="0,0,0,12">
                            <Grid.ColumnDefinitions>
                                <ColumnDefinition Width="*"/>
                                <ColumnDefinition Width="Auto"/>
                            </Grid.ColumnDefinitions>
                            <TextBox x:Name="TxtTargetDir" Grid.Column="0" Text="C:\TeatroGestion"/>
                            <Button x:Name="BtnBrowseTarget" Grid.Column="1" Content="Examinar..." Margin="8,0,0,0" Background="#2C2C38" Foreground="#FFFFFF"/>
                        </Grid>

                        <Grid Margin="0,0,0,12">
                            <Grid.ColumnDefinitions>
                                <ColumnDefinition Width="*"/>
                                <ColumnDefinition Width="*"/>
                            </Grid.ColumnDefinitions>
                            <!-- Duración -->
                            <StackPanel Grid.Column="0" Margin="0,0,8,0">
                                <TextBlock Text="Vigencia de la Licencia:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                                <ComboBox x:Name="CmbLicenseTerm" SelectedIndex="0">
                                    <ComboBoxItem Content="1 Año (365 días) — Recomendado" Tag="4"/>
                                    <ComboBoxItem Content="1 Mes (30 días)" Tag="1"/>
                                    <ComboBoxItem Content="3 Meses (90 días)" Tag="2"/>
                                    <ComboBoxItem Content="6 Meses (180 días)" Tag="3"/>
                                    <ComboBoxItem Content="Perpetua (Sin vencimiento)" Tag="5"/>
                                    <ComboBoxItem Content="Días personalizados…" Tag="6"/>
                                </ComboBox>
                            </StackPanel>
                            <!-- Puesto -->
                            <StackPanel Grid.Column="1" Margin="8,0,0,0">
                                <TextBlock Text="Puesto / Ubicación de esta PC:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                                <TextBox x:Name="TxtPuesto" Text="Cabina / Oficina"/>
                            </StackPanel>
                        </Grid>

                        <!-- Contraseña Maestra -->
                        <TextBlock Text="Contraseña Maestra (master.key.enc):" FontWeight="SemiBold" Margin="0,0,0,4"/>
                        <PasswordBox x:Name="TxtMasterPass" Margin="0,0,0,15"/>

                        <!-- Bóveda Section -->
                        <Border Background="#141418" BorderBrush="#33333F" BorderThickness="1" CornerRadius="6" Padding="12" Margin="0,0,0,15">
                            <StackPanel>
                                <TextBlock Text="Bóveda Documental de Inspección (Carina Lion)" FontWeight="Bold" Foreground="#B388FF" Margin="0,0,0,6"/>
                                <TextBlock Text="Proteja con contraseña los certificados médicos, facturas de caja chica y memos." FontSize="11" Foreground="#888899" Margin="0,0,0,10"/>
                                
                                <Grid Margin="0,0,0,10">
                                    <Grid.ColumnDefinitions>
                                        <ColumnDefinition Width="*"/>
                                        <ColumnDefinition Width="*"/>
                                    </Grid.ColumnDefinitions>
                                    <StackPanel Grid.Column="0" Margin="0,0,8,0">
                                        <TextBlock Text="Contraseña de la Inspectora:" Margin="0,0,0,4"/>
                                        <PasswordBox x:Name="TxtVaultPass"/>
                                    </StackPanel>
                                    <StackPanel Grid.Column="1" Margin="8,0,0,0">
                                        <TextBlock Text="Confirmar Contraseña:" Margin="0,0,0,4"/>
                                        <PasswordBox x:Name="TxtVaultPassConfirm"/>
                                    </StackPanel>
                                </Grid>

                                <TextBlock Text="Bloqueo automático por inactividad:" Margin="0,0,0,4"/>
                                <ComboBox x:Name="CmbVaultTimeout" SelectedIndex="1" Width="260" HorizontalAlignment="Left">
                                    <ComboBoxItem Content="1 minuto" Tag="1"/>
                                    <ComboBoxItem Content="5 minutos (Recomendado)" Tag="5"/>
                                    <ComboBoxItem Content="15 minutos" Tag="15"/>
                                    <ComboBoxItem Content="30 minutos" Tag="30"/>
                                    <ComboBoxItem Content="Nunca bloquear automáticamente" Tag="0"/>
                                </ComboBox>
                            </StackPanel>
                        </Border>

                        <Button x:Name="BtnInstall" Content="INSTALAR Y ACTIVAR TEATROGESTIÓN" Height="42" FontSize="14" FontWeight="Bold"/>
                    </StackPanel>
                </ScrollViewer>
            </TabItem>

            <!-- Tab 2: Recuperación de Bóveda -->
            <TabItem Header="Recuperar Contraseña de Bóveda">
                <ScrollViewer VerticalScrollBarVisibility="Auto" Padding="15">
                    <StackPanel>
                        <Border Background="#1E1A14" BorderBrush="#FFB31A" BorderThickness="1" CornerRadius="6" Padding="12" Margin="0,0,0,15">
                            <TextBlock Text="Esta herramienta permite al Administrador restablecer la clave de la Bóveda de Inspección si la inspectora la olvidó, validando la llave maestra del pendrive." FontSize="12" Foreground="#FFE082" TextWrapping="Wrap"/>
                        </Border>

                        <TextBlock Text="Ubicación de la Base de Datos SQLite:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                        <Grid Margin="0,0,0,12">
                            <Grid.ColumnDefinitions>
                                <ColumnDefinition Width="*"/>
                                <ColumnDefinition Width="Auto"/>
                            </Grid.ColumnDefinitions>
                            <TextBox x:Name="TxtResetDbPath" Grid.Column="0"/>
                            <Button x:Name="BtnBrowseDb" Grid.Column="1" Content="Examinar..." Margin="8,0,0,0" Background="#2C2C38" Foreground="#FFFFFF"/>
                        </Grid>

                        <TextBlock Text="Contraseña Maestra del Administrador:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                        <PasswordBox x:Name="TxtResetMasterPass" Margin="0,0,0,12"/>

                        <Grid Margin="0,0,0,12">
                            <Grid.ColumnDefinitions>
                                <ColumnDefinition Width="*"/>
                                <ColumnDefinition Width="*"/>
                            </Grid.ColumnDefinitions>
                            <StackPanel Grid.Column="0" Margin="0,0,8,0">
                                <TextBlock Text="Nueva Contraseña para la Inspectora:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                                <PasswordBox x:Name="TxtResetVaultPass"/>
                            </StackPanel>
                            <StackPanel Grid.Column="1" Margin="8,0,0,0">
                                <TextBlock Text="Confirmar Nueva Contraseña:" FontWeight="SemiBold" Margin="0,0,0,4"/>
                                <PasswordBox x:Name="TxtResetVaultPassConfirm"/>
                            </StackPanel>
                        </Grid>

                        <TextBlock Text="Tiempo de bloqueo por inactividad:" Margin="0,0,0,4"/>
                        <ComboBox x:Name="CmbResetVaultTimeout" SelectedIndex="1" Width="260" HorizontalAlignment="Left" Margin="0,0,0,15">
                            <ComboBoxItem Content="1 minuto" Tag="1"/>
                            <ComboBoxItem Content="5 minutos (Recomendado)" Tag="5"/>
                            <ComboBoxItem Content="15 minutos" Tag="15"/>
                            <ComboBoxItem Content="30 minutos" Tag="30"/>
                            <ComboBoxItem Content="Nunca bloquear automáticamente" Tag="0"/>
                        </ComboBox>

                        <Button x:Name="BtnResetVault" Content="RESTABLECER CONTRASEÑA DE BÓVEDA" Height="42" FontSize="14" FontWeight="Bold" Background="#FF9800"/>
                    </StackPanel>
                </ScrollViewer>
            </TabItem>
        </TabControl>

        <!-- Progress and Status -->
        <Grid Grid.Row="2" Margin="0,0,0,8">
            <Grid.ColumnDefinitions>
                <ColumnDefinition Width="*"/>
                <ColumnDefinition Width="Auto"/>
            </Grid.ColumnDefinitions>
            <ProgressBar x:Name="PrgProgress" Grid.Column="0" Height="8" Background="#18181E" Foreground="#FFB31A" BorderThickness="0" Margin="0,0,15,0"/>
            <CheckBox x:Name="ChkLaunchAfter" Grid.Column="1" Content="Iniciar TeatroGestión al terminar" IsChecked="True" Foreground="#CCCCCC" VerticalAlignment="Center"/>
        </Grid>

        <!-- Log Output -->
        <Border Grid.Row="3" Background="#0A0A0C" BorderBrush="#25252E" BorderThickness="1" CornerRadius="6" Padding="6">
            <TextBox x:Name="TxtLog" Background="Transparent" Foreground="#A5D6A7" BorderThickness="0" 
                     FontFamily="Consolas" FontSize="11" IsReadOnly="True" 
                     VerticalScrollBarVisibility="Auto" TextWrapping="Wrap"/>
        </Border>
    </Grid>
</Window>
"@

$reader = (New-Object System.Xml.XmlNodeReader $xaml)
$window = [System.Windows.Markup.XamlReader]::Load($reader)

# Elementos
$txtTargetDir = $window.FindName("TxtTargetDir")
$btnBrowseTarget = $window.FindName("BtnBrowseTarget")
$cmbLicenseTerm = $window.FindName("CmbLicenseTerm")
$txtPuesto = $window.FindName("TxtPuesto")
$txtMasterPass = $window.FindName("TxtMasterPass")
$txtVaultPass = $window.FindName("TxtVaultPass")
$txtVaultPassConfirm = $window.FindName("TxtVaultPassConfirm")
$cmbVaultTimeout = $window.FindName("CmbVaultTimeout")
$btnInstall = $window.FindName("BtnInstall")

$txtResetDbPath = $window.FindName("TxtResetDbPath")
$btnBrowseDb = $window.FindName("BtnBrowseDb")
$txtResetMasterPass = $window.FindName("TxtResetMasterPass")
$txtResetVaultPass = $window.FindName("TxtResetVaultPass")
$txtResetVaultPassConfirm = $window.FindName("TxtResetVaultPassConfirm")
$cmbResetVaultTimeout = $window.FindName("CmbResetVaultTimeout")
$btnResetVault = $window.FindName("BtnResetVault")

$prgProgress = $window.FindName("PrgProgress")
$chkLaunchAfter = $window.FindName("ChkLaunchAfter")
$txtLog = $window.FindName("TxtLog")

function Log-Message([string]$msg) {
    $timestamp = (Get-Date).ToString("HH:mm:ss")
    $txtLog.AppendText("[$timestamp] $msg`r`n")
    $txtLog.ScrollToEnd()
    [System.Windows.Forms.Application]::DoEvents()
}

# Inicializar ruta detectada de SQLite para Reseteo
$appdata = [Environment]::GetFolderPath("ApplicationData")
$defaultSqlite = Join-Path $appdata "com.teatrogestion.app\teatro_gestion.sqlite"
$txtResetDbPath.Text = $defaultSqlite

Log-Message "Instalador de TeatroGestión iniciado."
Log-Message "Ubicación del instalador nativo: $installerExe"

# Examinar Carpeta Destino
$btnBrowseTarget.Add_Click({
    $dlg = New-Object System.Windows.Forms.FolderBrowserDialog
    $dlg.SelectedPath = $txtTargetDir.Text
    if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        $txtTargetDir.Text = $dlg.SelectedPath
    }
})

# Examinar SQLite
$btnBrowseDb.Add_Click({
    $dlg = New-Object System.Windows.Forms.OpenFileDialog
    $dlg.Filter = "Base SQLite (*.sqlite;*.db)|*.sqlite;*.db|Todos los archivos (*.*)|*.*"
    if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        $txtResetDbPath.Text = $dlg.FileName
    }
})

# Botón Instalar
$btnInstall.Add_Click({
    $btnInstall.IsEnabled = $false
    $prgProgress.IsIndeterminate = $true

    # Sin barra final («C:\TeatroGestion\» entre comillas escapa la comilla de cierre) y sin comillas en el puesto.
    $target = $txtTargetDir.Text.Trim().TrimEnd('\')
    $puesto = $txtPuesto.Text.Trim().Replace('"', "'")
    $masterPass = $txtMasterPass.Password
    $termTag = $cmbLicenseTerm.SelectedItem.Tag
    $customDays = 365
    $vaultPass = $txtVaultPass.Password
    $vaultPassConf = $txtVaultPassConfirm.Password
    $vaultTimeout = $cmbVaultTimeout.SelectedItem.Tag

    if ($vaultPass -ne "" -and $vaultPass -ne $vaultPassConf) {
        [System.Windows.MessageBox]::Show("Las contraseñas de la Bóveda de Inspección no coinciden.", "Error de Validación", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Warning)
        $btnInstall.IsEnabled = $true
        $prgProgress.IsIndeterminate = $false
        return
    }

    # «Días personalizados»: antes siempre quedaba en 365 sin preguntar.
    if ($termTag -eq "6") {
        $daysInput = [Microsoft.VisualBasic.Interaction]::InputBox("Cantidad de días de vigencia (1 a 3650):", "Días de la licencia", "365")
        $parsed = 0
        if (-not [int]::TryParse($daysInput, [ref]$parsed) -or $parsed -lt 1 -or $parsed -gt 3650) {
            [System.Windows.MessageBox]::Show("Ingrese un número de días entre 1 y 3650.", "Días de la licencia", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Warning)
            $btnInstall.IsEnabled = $true
            $prgProgress.IsIndeterminate = $false
            return
        }
        $customDays = $parsed
    }

    Log-Message "Iniciando instalación en: $target"
    Log-Message "Generando licencia (Opción $termTag) para puesto '$puesto'..."

    # Invocación a Instalar_TeatroGestion.exe --silent-install
    $pinfo = New-Object System.Diagnostics.ProcessStartInfo
    $pinfo.FileName = $installerExe
    # Las contraseñas viajan por variables de entorno del proceso hijo, NO por la línea de comandos (que cualquier programa
    # puede leer en la lista de procesos).
    $pinfo.Arguments = "--silent-install `"$target`" $termTag $customDays `"$puesto`" $vaultTimeout"
    $pinfo.EnvironmentVariables["TEATROGESTION_MASTER_PASSWORD"] = $masterPass
    if ($vaultPass -ne "") { $pinfo.EnvironmentVariables["TEATROGESTION_VAULT_PASSWORD"] = $vaultPass }
    $pinfo.RedirectStandardOutput = $true
    $pinfo.RedirectStandardError = $true
    $pinfo.UseShellExecute = $false
    $pinfo.CreateNoWindow = $true

    $p = New-Object System.Diagnostics.Process
    $p.StartInfo = $pinfo

    try {
        $p.Start() | Out-Null
        $stdout = $p.StandardOutput.ReadToEnd()
        $stderr = $p.StandardError.ReadToEnd()
        $p.WaitForExit()

        if ($stdout) {
            foreach ($line in ($stdout -split "`r?`n")) {
                if ($line.Trim()) { Log-Message $line }
            }
        }
        if ($stderr) {
            foreach ($line in ($stderr -split "`r?`n")) {
                if ($line.Trim()) { Log-Message "[!] $line" }
            }
        }

        if ($p.ExitCode -eq 0) {
            $prgProgress.IsIndeterminate = $false
            $prgProgress.Value = 100
            Log-Message "¡INSTALACIÓN COMPLETADA EXITOSAMENTE!"
            [System.Windows.MessageBox]::Show("TeatroGestión ha sido instalado y activado exitosamente.`n`nUbicación: $target", "Instalación Exitosa", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Information)

            if ($chkLaunchAfter.IsChecked) {
                $exePath = Join-Path $target "TeatroGestion.exe"
                if (Test-Path $exePath) {
                    Log-Message "Lanzando TeatroGestión..."
                    Start-Process $exePath -WorkingDirectory $target
                }
            }
        } else {
            Log-Message "La instalación finalizó con código de error $($p.ExitCode)."
            [System.Windows.MessageBox]::Show("Ocurrió un error durante la instalación. Revise los registros en pantalla.", "Error de Instalación", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Error)
        }
    } catch {
        Log-Message "Excepción al ejecutar instalador: $_"
    } finally {
        $btnInstall.IsEnabled = $true
        $prgProgress.IsIndeterminate = $false
    }
})

# Botón Resetear Bóveda
$btnResetVault.Add_Click({
    $btnResetVault.IsEnabled = $false
    $prgProgress.IsIndeterminate = $true

    $dbPath = $txtResetDbPath.Text.Trim()
    $masterPass = $txtResetMasterPass.Password
    $newVaultPass = $txtResetVaultPass.Password
    $newVaultPassConf = $txtResetVaultPassConfirm.Password
    $timeout = $cmbResetVaultTimeout.SelectedItem.Tag

    if ($newVaultPass -eq "") {
        [System.Windows.MessageBox]::Show("La nueva contraseña de la Bóveda no puede estar vacía.", "Validación", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Warning)
        $btnResetVault.IsEnabled = $true
        $prgProgress.IsIndeterminate = $false
        return
    }

    if ($newVaultPass -ne $newVaultPassConf) {
        [System.Windows.MessageBox]::Show("Las contraseñas no coinciden.", "Validación", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Warning)
        $btnResetVault.IsEnabled = $true
        $prgProgress.IsIndeterminate = $false
        return
    }

    Log-Message "Iniciando restablecimiento administrativo de Bóveda en: $dbPath"

    $pinfo = New-Object System.Diagnostics.ProcessStartInfo
    $pinfo.FileName = $installerExe
    $pinfo.Arguments = "--silent-reset-vault `"$dbPath`" $timeout"
    $pinfo.EnvironmentVariables["TEATROGESTION_MASTER_PASSWORD"] = $masterPass
    $pinfo.EnvironmentVariables["TEATROGESTION_NEW_VAULT_PASSWORD"] = $newVaultPass
    $pinfo.RedirectStandardOutput = $true
    $pinfo.RedirectStandardError = $true
    $pinfo.UseShellExecute = $false
    $pinfo.CreateNoWindow = $true

    $p = New-Object System.Diagnostics.Process
    $p.StartInfo = $pinfo

    try {
        $p.Start() | Out-Null
        $stdout = $p.StandardOutput.ReadToEnd()
        $stderr = $p.StandardError.ReadToEnd()
        $p.WaitForExit()

        if ($stdout) {
            foreach ($line in ($stdout -split "`r?`n")) {
                if ($line.Trim()) { Log-Message $line }
            }
        }
        if ($stderr) {
            foreach ($line in ($stderr -split "`r?`n")) {
                if ($line.Trim()) { Log-Message "[!] $line" }
            }
        }

        if ($p.ExitCode -eq 0) {
            Log-Message "¡CONTRASEÑA DE BÓVEDA RESTABLECIDA CON ÉXITO!"
            [System.Windows.MessageBox]::Show("La contraseña de la Bóveda de Inspección ha sido restablecida exitosamente en la base de datos.", "Bóveda Restablecida", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Information)
        } else {
            [System.Windows.MessageBox]::Show("No se pudo restablecer la contraseña. Verifique que la contraseña maestra del desarrollador sea correcta.", "Error de Autenticación", [System.Windows.MessageBoxButton]::OK, [System.Windows.MessageBoxImage]::Error)
        }
    } catch {
        Log-Message "Excepción al restablecer contraseña: $_"
    } finally {
        $btnResetVault.IsEnabled = $true
        $prgProgress.IsIndeterminate = $false
    }
})

$window.ShowDialog() | Out-Null
